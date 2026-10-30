if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Rabbit AI",
		desc	= "Implements movement, eating and running for Rabbits",
		author	= "Google Frog",
		date	= "8 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 10,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Configs/Speedups

local sqrt = math.sqrt
local spValidUnitID = Spring.ValidUnitID
local spGetGameFrame = Spring.GetGameFrame
local spGetUnitPosition = Spring.GetUnitPosition
local spGetUnitVelocity = Spring.GetUnitVelocity
local spGetHeadingFromVector = Spring.GetHeadingFromVector
local spSetUnitMoveGoal = Spring.SetUnitMoveGoal
local spSetUnitRulesParam = Spring.SetUnitRulesParam
local spGetUnitRulesParam = Spring.GetUnitRulesParam
local spGetUnitIsDead = Spring.GetUnitIsDead
local spGetAllUnits = Spring.GetAllUnits
local spGetUnitDefID = Spring.GetUnitDefID
local spSetGameRulesParam = Spring.SetGameRulesParam
local spGetGameRulesParam = Spring.GetGameRulesParam
local spDestroyUnit = Spring.DestroyUnit
local math_random = math.random
local math_floor = math.floor
local math_min = math.min
local math_max = math.max
local math_cos = math.cos
local math_sin = math.sin
local math_pi = math.pi

local rabbitUnitDef = UnitDefs[UnitDefNames["rabbit"].id]

local rabbitDefID = {
	[UnitDefNames["rabbit"].id] = true
}

local desirableUnitDefs = {
	[UnitDefNames["carrot"].id] = {
		radius = 1500,
		radiusSq = 1500^2,
		edgeMagnitude = 0.1, -- Magnitude once within radius (per frame)
		proximityMagnitude = 2, -- Maximum agnitude gained by being close (per frame)
		thingType = 1, -- Food
		eatTime = 100,
		eatTimeReduces = true,
		isEdible = true
	},
	[UnitDefNames["carrot_dropped"].id] = {
		radius = 1500,
		radiusSq = 1500^2,
		edgeMagnitude = 0.1, -- Magnitude once within radius (per frame)
		proximityMagnitude = 2.15, -- Maximum agnitude gained by being close (per frame)
		thingType = 1, -- Food
		eatTime = 10,
		isEdible = true,
	},
	[UnitDefNames["mine"].id] = {
		radius = 1500,
		radiusSq = 1500^2,
		edgeMagnitude = 0.1, -- Magnitude once within radius (per frame)
		proximityMagnitude = 2, -- Maximum agnitude gained by being close (per frame)
		thingType = 1, -- "Food"
	},
	[UnitDefNames["burrow"].id] = {
		radius = 9000,
		radiusSq = 9000^2,
		edgeMagnitude = 1, -- Magnitude once within radius (per frame)
		proximityMagnitude = 2, -- Maximum magnitude gained by being close (per frame)
		thingType = 2, -- Safety
		isCarrotRepository = true,
	},
}

-- Rabbit fear goes down by (1 - FEAR_DECAY)*100% per frame.
-- For example 10 seconds after having 100 fear a rabbit will
-- have 100*0.995^300 = 22.2 fear
local FEAR_DECAY = 0.995
local STAMINA_DECAY = 0.998
local fearDecays, staminaDecays = {}, {}
for gap = 0, 15 do
    fearDecays[gap], staminaDecays[gap] = FEAR_DECAY^gap, STAMINA_DECAY^gap
end

local FEAR_ADDED = 0.3 -- fear added every frame
local BOLDNESS_ADDED = 0.3 -- boldness added every frame

local global_rabbitSpeedMult = 1
local global_rabbitPanicResist = 1

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Global Tables
--[[
scaryThings are locations which scare rabbits. For example:
 * Open fields (being exposed is a bit scary)
 * Sprung traps (proximity to visible traps, especially inhabited one, is scary)
 * Torch light (torch light is temorarilly a bit scary)
 * Recent shotgun locations (shotgun is very scary)
--]]
local scaryThings = {}
-- {[1] = {x, z, attributes = {radius, radiusSq, edgeMagnitude, proximityMagnitude, thingType}), [2] = {...}, ...}

-- Magnitude felt by rabbits is proximityMagnitude * (1 - distance/radius) + edgeMagnitude

--[[
desirableThings are things which rabbits want. This will be a burrow or carrot
depending on how hungry/scared a rabbit is. Some desirable things:
 * Carrot (short ranged food desirability)
 * Field (long ranged food desirability)
 * Burrow (safety desirability)
--]]
local desirableThings = {}
-- {[1] = {x, z, attributes = {radius, radiusSq, edgeMagnitude, proximityMagnitude, thingType}}, [2] = {...}, ...}

local desirableUnits = {}
local rabbits = {}
local rabbitOrder, rabbitOrderDirty = {}, true
local function OrderedRabbits()
    if rabbitOrderDirty then
        rabbitOrder = {}
        for id in pairs(rabbits) do rabbitOrder[#rabbitOrder+1] = id end
        table.sort(rabbitOrder)
        rabbitOrderDirty = false
    end
    return rabbitOrder
end
local FOOD_CELL_SIZE = 1500
local hungryDesirables, carryingDesirables = {1, 0}, {0, 1}
local carryingScares = {1, 1}
local foodCells = {}
local globalDesirables = {}

-- Rabbit AI is created by taking into account only the closest scary and desirable thing.

-------------------------------------------------------------------
-------------------------------------------------------------------
-- 2D Vector Functions

local function DistSq(x1,z1,x2,z2)
	return (x1 - x2)*(x1 - x2) + (z1 - z2)*(z1 - z2)
end

local function AbsVal(x, z)
	return sqrt(x*x + z*z)
end

-- Keep temporary vectors in numbers rather than allocating a table for every
-- normalization, rotation and addition in every rabbit's AI update.
local function Norm(b, x, z)
	local mag = AbsVal(x, z)
	if mag > 0 then
		return b*x/mag, b*z/mag
	else
		return x, z
	end
end

local function Angle(x, z)
	return -spGetHeadingFromVector(x, z)/2^15*math_pi + math_pi/2
end

local function PolarToCart(mag, dir)
	return mag*math_cos(dir), mag*math_sin(dir)
end


-------------------------------------------------------------------
-------------------------------------------------------------------
-- Movement Functions

local mapWidth = Game.mapSizeX
local mapHeight = Game.mapSizeZ
local spGiveOrderToUnit = Spring.GiveOrderToUnit
local CMD_INSERT = CMD.INSERT

function IsValidPosition(x, z)
	return x and z and x >= 1 and z >= 1 and x <= mapWidth-1 and z <= mapHeight-1
end

function ClampPosition(x, z)
	if x and z then
		if IsValidPosition(x, z) then
			return x, z
		else
			if x < 1 then
				x = 1
			elseif x > mapWidth-1 then
				x = mapWidth-1
			end
			if z < 1 then
				z = 1
			elseif z > mapHeight-1 then
				z = mapHeight-1
			end
			return x, z
		end
	end
end

function GiveClampedOrderToUnit(unitID, cmdID, params, options)
	local x, z = ClampPosition(params[1], params[3])
	spSetUnitMoveGoal(unitID, x, params[2], z, 16, nil, true) -- The last argument is whether the goal is raw
	--Spring.GiveOrderToUnit(unitID, cmdID, {x, params[2], z}, options)
	return true
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Thing Table Handling

local function GetThingAdjustedMagnitude(distance, attributes, typeMultiplier)
	local mag = attributes.proximityMagnitude * (1 - distance/attributes.radius) + attributes.edgeMagnitude
	if typeMultiplier and attributes.thingType and typeMultiplier[attributes.thingType] then
		mag = mag * typeMultiplier[attributes.thingType]
	end
	return mag
end
 
local function GetBestThing(thingTable, x, z, typeMultiplier)
	local bestIndex = 0
	local bestMagnitude = 0
	local bestX = 0
	local bestZ = 0
	-- Things sharing an attribute table have the same distance falloff. Once
	-- a nearer one was considered, a farther one cannot beat it. Small light
	-- lists use the simple loop without allocating a search cache.
	local closest = #thingTable > 32 and {}
	
	local thing, distanceSquare, distance, thingAtt, adjustedMagnitude
	
	for i = 1, #thingTable do
		thing = thingTable[i]
		if not thing.inactive then
			thingAtt = thing.attributes
			-- A carrying rabbit ignores food; a hungry one ignores burrows.
			-- Multiplying by zero can never beat the positive best magnitude.
			local multiplier = typeMultiplier and thingAtt.thingType and typeMultiplier[thingAtt.thingType]
			distanceSquare = multiplier ~= 0 and DistSq(x, z, thing.x, thing.z)
			local monotone = thingAtt.proximityMagnitude >= 0 and not (multiplier and multiplier < 0)
			if distanceSquare and distanceSquare < thingAtt.radiusSq
					and (not closest or not monotone or not closest[thingAtt] or distanceSquare < closest[thingAtt]) then
				if closest and monotone then
					closest[thingAtt] = distanceSquare
				end
				distance = sqrt(distanceSquare)
				adjustedMagnitude = GetThingAdjustedMagnitude(distance, thingAtt, typeMultiplier)
				if adjustedMagnitude > bestMagnitude then
					bestIndex = i
					bestMagnitude = adjustedMagnitude
					bestX = thing.x
					bestZ = thing.z
				end
			end
		end
	end
	
	if bestMagnitude > 0 then
		return thingTable[bestIndex], bestX, bestZ, bestMagnitude
	else
		return false, x, z, 0
	end
end

local function AddThing(thingTable, data)
	thingTable[#thingTable + 1] = data
	thingTable[#thingTable].index = #thingTable
	return #thingTable
end

local function RemoveThing(thingTable, index)
	thingTable[index] = thingTable[#thingTable]
	thingTable[index].index = index
	thingTable[#thingTable] = nil
	--GG.TableEcho(thingTable)
end

local function IndexDesirable(thing, isUnit)
	local list, indexKey
	if isUnit and thing.attributes.thingType == 1 and thing.attributes.radius <= FOOD_CELL_SIZE then
		local x, z = math_floor(thing.x/FOOD_CELL_SIZE), math_floor(thing.z/FOOD_CELL_SIZE)
		foodCells[x] = foodCells[x] or {}
		foodCells[x][z] = foodCells[x][z] or {}
		list = foodCells[x][z]
		indexKey = "foodIndex"
	else
		list, indexKey = globalDesirables, "globalIndex"
	end
	thing.searchList, thing.searchIndexKey = list, indexKey
	list[#list + 1] = thing
	thing[indexKey] = #list
end

local function UnindexDesirable(thing)
	local list, key = thing.searchList, thing.searchIndexKey
	local index = thing[key]
	list[index] = list[#list]
	list[index][key] = index
	list[#list] = nil
end

-- Keep the query state in arguments rather than allocating a new closure for
-- every rabbit. Attribute tables can be changed by other gadgets, so pruning
-- is restricted to non-increasing, non-negative distance falloff.
local desirableClosest = setmetatable({}, {__mode = "k"})
local desirableQuery = 0

local function ScanDesirables(list, x, z, typeMultiplier, best, bestMagnitude)
    for i = 1, #list do
        local thing = list[i]
        local attributes = thing.attributes
        local multiplier = typeMultiplier[attributes.thingType]
        if not thing.inactive and multiplier ~= 0 then
            local square = DistSq(x, z, thing.x, thing.z)
            if square < attributes.radiusSq then
                local previous = desirableClosest[attributes]
                local fresh = previous and previous.query == desirableQuery
                local monotone = attributes.proximityMagnitude >= 0 and not (multiplier and multiplier < 0)
                -- Equal computed magnitudes retain the first main-list entry.
                -- A farther candidate with an earlier index must still be
                -- evaluated, including floating-point falloff ties.
                if not monotone or not fresh or square < previous.square or thing.index < previous.index then
                    if monotone and (not fresh or square < previous.square or
                            (square == previous.square and thing.index < previous.index)) then
                        if previous then
                            previous.query, previous.square, previous.index = desirableQuery, square, thing.index
                        else
                            desirableClosest[attributes] = {query = desirableQuery, square = square, index = thing.index}
                        end
                    end
                    local magnitude = GetThingAdjustedMagnitude(sqrt(square), attributes, typeMultiplier)
                    if magnitude > bestMagnitude or
                            (best and magnitude == bestMagnitude and thing.index < best.index) then
                        best, bestMagnitude = thing, magnitude
                    end
                end
            end
        end
    end
    return best, bestMagnitude
end

local function GetBestDesirable(x, z, typeMultiplier)
    if #desirableThings <= 32 then
        return GetBestThing(desirableThings, x, z, typeMultiplier)
    end
    -- Lua_Number is float32 in the engine: keep generations exactly representable.
    if desirableQuery == 1048576 then
        desirableClosest = setmetatable({}, {__mode = "k"})
        desirableQuery = 0
    end
    desirableQuery = desirableQuery + 1
    local best, bestMagnitude = nil, 0
    best, bestMagnitude = ScanDesirables(globalDesirables, x, z, typeMultiplier, best, bestMagnitude)
    if typeMultiplier[1] ~= 0 then
        local cx, cz = math_floor(x/FOOD_CELL_SIZE), math_floor(z/FOOD_CELL_SIZE)
        for gx = cx - 1, cx + 1 do
            local row = foodCells[gx]
            if row then
                for gz = cz - 1, cz + 1 do
                    local list = row[gz]
                    if list then
                        best, bestMagnitude = ScanDesirables(list, x, z, typeMultiplier, best, bestMagnitude)
                    end
                end
            end
        end
    end
    if best then return best, best.x, best.z, bestMagnitude end
    return false, x, z, 0
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Purea Scary/Desirable Area Handling
-- Areas can be created by external gadget. Those gadgets are
-- allowed to update the attributes of the area.

local function AddScaryArea(data)
	local index = AddThing(scaryThings, data)
	return scaryThings[index]
end

local function RemoveScaryArea(data)
	RemoveThing(scaryThings, data.index)
end
local function AddScaryArea(data)
	local index = AddThing(scaryThings, data)
	return scaryThings[index]
end

local function AddDesirableArea(data)
	local index = AddThing(desirableThings, data)
	IndexDesirable(data, false)
	return desirableThings[index]
end

local function RemoveDesirableArea(data)
	UnindexDesirable(data)
	RemoveThing(desirableThings, data.index)
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Desirable Unit Handling

local function StartStealing(rabbitData, thingRef)
	if spValidUnitID(rabbitData.unitID) then
		spSetUnitRulesParam(rabbitData.unitID, "stealing", 1)
	end
	
	--Spring.MarkerAddPoint(thingRef.x, 0, thingRef.z, "Stealing")
	
	rabbitData.eatingProgress = thingRef.carryoverEatProgress or 0
	rabbitData.eatingThingRef = thingRef
	
	local env = Spring.UnitScript.GetScriptEnv(thingRef.unitID)
	if env and env.StartBeingStolen then
		Spring.UnitScript.CallAsUnit(thingRef.unitID, env.StartBeingStolen, rabbitData.eatingProgress)
	end

	thingRef.inactive = true
	thingRef.eater = rabbitData
end

local function StopStealing(rabbitData)
	if spValidUnitID(rabbitData.unitID) then
		spSetUnitRulesParam(rabbitData.unitID, "stealing", 0)
	end
	
	local thingRef = rabbitData.eatingThingRef
	if thingRef.attributes.eatTimeReduces then
		thingRef.carryoverEatProgress = rabbitData.eatingProgress
	end
	thingRef.inactive = false
	thingRef.eater = nil
	
	local env = Spring.UnitScript.GetScriptEnv(thingRef.unitID)
	if env and env.StopBeingStolen then
		Spring.UnitScript.CallAsUnit(thingRef.unitID,env.StopBeingStolen, thingRef.carryoverEatProgress or 0)
	end
	
	rabbitData.eatingProgress = false
	rabbitData.eatingThingRef = nil
end

local function AddDesirableUnit(unitID, unitDefID)
	local x,_,z = spGetUnitPosition(unitID)
	local attributes = desirableUnitDefs[unitDefID]
	local index = AddThing(desirableThings, {
		x = x,
		z = z,
		unitID = unitID,
		attributes = attributes,
	})
	
	desirableUnits[unitID] = {thingTableEntry = desirableThings[index]}
	IndexDesirable(desirableThings[index], true)
end

local function RemoveDesirableUnit(unitID)
	local thingRef = desirableUnits[unitID].thingTableEntry
	if thingRef.eater then
		StopStealing(thingRef.eater)
	end

	local index = thingRef.index
	UnindexDesirable(thingRef)
	RemoveThing(desirableThings, index)
	desirableUnits[unitID] = nil
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Rabbit Handling

local function AddRabbit(unitID)
	rabbitOrderDirty = true
	rabbits[unitID] = {
		unitID = unitID,
		fear = 50,
		boldness = 150,
		stamina = 100,
		foodCarried = 0,
		carryingSince = false,
		lastUpdate = spGetGameFrame(),
	}
end

local function RemoveRabbit(unitID)
    -- TODO: this was put to reduce spam
    if rabbits[unitID] == nil then
        return
    end
	if rabbits[unitID].foodCarried > 0 then
		GG.RabbitDropCarrot(unitID)
	end
	
	if rabbits[unitID].eatingProgress then
		StopStealing(rabbits[unitID])
	end
	rabbits[unitID] = nil
	rabbitOrderDirty = true
	
	if not spGetUnitRulesParam(unitID, "internalDestroy") then
		local killed = spGetGameRulesParam("rabbits_killed")
		spSetGameRulesParam("rabbits_killed", killed + 1)
		local score = spGetGameRulesParam("score") or 0
		spSetGameRulesParam("score", score + 50)
	end
end

local function RabbitFoodDestroyed(unitID)
	local rabbitData = rabbits[unitID]
	if rabbitData and rabbitData.foodCarried then
		rabbitData.foodCarried = 0
		rabbitData.carryingSince = false
	end
end

local function SetRabbitMovement(unitID, x, z, goalX, goalZ, speedMult, accelMult, turnMult, frame)
	spSetUnitRulesParam(unitID, "selfMoveSpeedChange", speedMult)
		spSetUnitRulesParam(unitID, "selfMaxAccelerationChange", accelMult)
	spSetUnitRulesParam(unitID, "selfTurnSpeedChange", turnMult)
	GG.UpdateUnitAttributes(unitID, frame, speedMult, turnMult, accelMult, rabbitUnitDef)
	local targetX, targetZ = ClampPosition(goalX + x, goalZ + z)
	spSetUnitMoveGoal(unitID, targetX, 0, targetZ, 16, nil, true)
end

local function UpdateRabbit(unitID, frame, scaryOverride)

	if not rabbits[unitID] then
		return
	end
	
	--// Collect rabbit information
	frame = frame or spGetGameFrame()
	local x,_,z = spGetUnitPosition(unitID)
	if not x then return end
	local rabbitData = rabbits[unitID]

	local updateGap = frame - rabbitData.lastUpdate
	rabbitData.lastUpdate = frame
	rabbitData.nextUpdate = frame + 5 + 10*math_random()
	
	-- ScaryThingTable
	local scaryThingTable
	if rabbitData.carryingSince then
		local carrySeconds = (frame - rabbitData.carryingSince)/30
		-- After 10 seconds of carrying the Rabbit begins to ignore lights.
		-- After 20 seconds it ignores lights entirely.
        carryingScares[2] = math_min(1, math_max(0, (20 - carrySeconds)/10))
        scaryThingTable = carryingScares
	end
	
	--// Update Scary Place and Fear
	local scaryRef, sX, sZ, scaryMag, scaryFear
	if scaryOverride then
		-- This type of fear is from a sudden event (shotgun?). It is not
		-- multiplied by time since last update because it is treated as
		-- instantaneous.
		sX, sZ, scaryMag = scaryOverride[1], scaryOverride[2], scaryOverride[3]
		scaryFear = scaryMag
	else
		-- This type of fear is due to constant environmental effects
		-- so it is multiplied by the time since last update.
		-- These fears should be set low.
		scaryRef, sX, sZ, scaryMag = GetBestThing(scaryThings, x, z, scaryThingTable)
		scaryFear = scaryMag*updateGap
	end
	
    local fear = rabbitData.fear
	-- Panic mode causes a rabbit to run away from a scary location until it calms down.
	if rabbitData.panicMode and (scaryMag + 80*global_rabbitPanicResist > fear or scaryMag > 80*global_rabbitPanicResist) then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end
	
	-- Update Fear
	fear = (fear + scaryFear + FEAR_ADDED*updateGap)*((fearDecays[updateGap] or FEAR_DECAY^updateGap))
    rabbitData.fear = fear
	
	if rabbitData.panicMode then
		if fear < 150 then
			rabbitData.panicMode = false
		end
	elseif fear > 250*global_rabbitPanicResist then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end
	
	-- Paniced Rabbits Drop Carrots
	if rabbitData.fear > 500*global_rabbitPanicResist and rabbitData.foodCarried > 0 then
		rabbitData.foodCarried = 0
		rabbitData.carryingSince = false
		GG.RabbitDropCarrot(unitID)
	end
	
    local fear, boldness, stamina = rabbitData.fear, rabbitData.boldness, rabbitData.stamina
	--// Update Goal and Boldness
	local expectedFood = rabbitData.foodCarried + ((rabbitData.eatingProgress and 1) or 0)
	-- 1 carrot
    local deisrableTypeTable = expectedFood > 0 and carryingDesirables or hungryDesirables
	
	local goalRef, gX, gZ, goalMag = GetBestDesirable(x, z, deisrableTypeTable)
	
	boldness = (boldness + goalMag*updateGap + BOLDNESS_ADDED*updateGap)*
		(1/(0.99 + fear/(10000 - math_min(8000, boldness*20))))^updateGap

    rabbitData.boldness = boldness

	--// Update Speed and Stamina
	local speedMult = ((((fear + math_max(0, boldness - 320))/45)^0.8)*(2 + (boldness/200)^0.45)/2.3)*stamina/150

	-- High scaryMag means that something scary is happening right now!
	-- (Suddden blast or near center of a torch)
	speedMult = speedMult*math_max(1, math_min(2, (scaryMag - 0.7)*2))
	
	stamina = (stamina - ((speedMult)^0.2)*updateGap + 1.3*updateGap)*(staminaDecays[updateGap] or STAMINA_DECAY^updateGap)
	
    rabbitData.stamina = stamina
	speedMult = speedMult*1.8*global_rabbitSpeedMult
	
	--// Handle Carrot Eating
	-- Eat until the carrot is gone. If paniced stop eating and run away.
	if rabbitData.eatingProgress then
		if rabbitData.panicMode then
			StopStealing(rabbitData)
		else
			rabbitData.eatingProgress = rabbitData.eatingProgress + updateGap
			if rabbitData.eatingProgress > rabbitData.eatingThingRef.attributes.eatTime then
				GG.RabbitPickupCarrot(unitID)
				local carrotID = rabbitData.eatingThingRef.unitID
				-- Release the reservation before DestroyUnit raises UnitDestroyed.
				StopStealing(rabbitData)
				spSetUnitRulesParam(carrotID, "internalDestroy", 1)
				spDestroyUnit(carrotID, false, false)
				rabbitData.foodCarried = rabbitData.foodCarried + 1
				rabbitData.boldness = rabbitData.boldness + 150 -- Bonus boldness for being a good theif!
				rabbitData.carryingSince = frame
			else
				SetRabbitMovement(unitID, x, z, rabbitData.eatingThingRef.x - x, rabbitData.eatingThingRef.z - z, 0.05, 2, 0.2, frame)
				return
			end
		end
	end
	
	--speedMult = math_min(100, speedMult^5)
	
	--GG.UnitEcho(unitID, math_floor(rabbitData.fear))
	--Spring.Echo("Fear", rabbitData.fear)
	--Spring.Echo("Boldness", rabbitData.boldness)
	--Spring.Echo("Stamina", rabbitData.stamina)
	--Spring.Echo("speedMult", speedMult)
	
    local fear, boldness = rabbitData.fear, rabbitData.boldness
	--// Determine move direction and speed
	-- scaryVec is the direction and magnitude to the scariest thing.
	-- goalVec is the same for the goal
	local scaryX, scaryZ
	if rabbitData.panicMode then
		scaryX, scaryZ = rabbitData.panicMode.x - x, rabbitData.panicMode.z - z
	else
		scaryX, scaryZ = sX - x, sZ - z
	end
	
	-- These vectors are scaled by fear and boldness
	-- If fear is too high then the goal is ignored.
	-- moveVec is the resultant move direction from fear and boldness.

	
	scaryX, scaryZ = Norm(-fear/8, scaryX, scaryZ)
	
	local moveX, moveZ, goalMag
	if fear < 180*global_rabbitPanicResist + 150*rabbitData.foodCarried then
		local goalX, goalZ = gX - x, gZ - z
		goalMag = AbsVal(goalX, goalZ)
		if goalMag < 200 then
			speedMult = speedMult*1.2
			goalX, goalZ = Norm(boldness/10, goalX, goalZ)
		else
			goalX, goalZ = Norm(boldness/30, goalX, goalZ)
		end
		moveX, moveZ = scaryX + goalX, scaryZ + goalZ
	else
		moveX, moveZ = scaryX, scaryZ
	end 
	
	--// Handle Transition into Non-Moving (eating, depositing etc..)
	-- Pick up a nearby carrot
	if goalRef and goalMag and goalMag < 60 and goalRef.attributes.isEdible and 
			(not rabbitData.eatingProgress) and rabbitData.foodCarried == 0 and (not rabbitData.panicMode) then
		StartStealing(rabbitData, goalRef)
		rabbitData.eatingProgress = rabbitData.eatingProgress + updateGap
		SetRabbitMovement(unitID, x, z, gX - x, gZ - z, 0.05, 2, 0.2, frame)
		return
	end
	
	-- Deposit a carrot in a burrow
	if goalRef and goalMag and goalMag < 60 and goalRef.attributes.isCarrotRepository and 
			rabbitData.foodCarried > 0 and (not rabbitData.panicMode) then
		rabbitData.foodCarried = 0
		rabbitData.carryingSince = false
		GG.RabbitScoreCarrot(unitID)
		SetRabbitMovement(unitID, x, z, gX - x, gZ - z, 0.05, 2, 1, frame)
		return
	end
	
	--// Normal Movement
	-- At this point moveVec is the direction which a rabbit would go if it were bold.
	-- Direction is 0 in positive x direction. Increases clockwise.
	local moveDir = Angle(moveX, moveZ)
	local moveMag = AbsVal(moveX, moveZ)
	
	--Spring.MarkerAddPoint(moveVec[1], 0 ,moveVec[2])
	--Spring.Echo("Move", moveVec[1], moveVec[2], moveDir)
	
	-- Boldness decreases rabbit movement jitter.
	local dirRandomness = math_min(math_pi*0.4, 20*(10 + boldness)^(-0.9))
	local moveDir = moveDir + math_random()*2*dirRandomness - dirRandomness
	
	local jitter = 1/(1 + math_max(0, boldness - 300)/100)
	
	-- Rabbits have a small bias towards keeping their momentum
	local vx, _, vz, velMag = spGetUnitVelocity(unitID)
	local velX, velZ = Norm((10 + math_random()*5)*jitter, vx, vz)
	moveX, moveZ = PolarToCart(moveMag, moveDir)
	
	-- Rabbits also just move randomly.
	local randX, randZ = PolarToCart((5 + math_random()*10)*jitter, math_random()*2*math_pi)
	
	--Spring.Echo("jitter", jitter)
	--Spring.Echo("moveVec Angle", Angle(moveVec)*180/math_pi)
	--Spring.Echo("randVec", AbsVal(randVec))
	--Spring.Echo("dirRandomness", dirRandomness*180/math_pi)
	
	-- Add the randomVector, momentumVector and goal+scary vectors to get final direction.
	moveX, moveZ = Norm(200*speedMult, moveX + (randX + velX), moveZ + (randZ + velZ))
	
	--// Modify movement attributes and goal
	if scaryMag > 80 then
		SetRabbitMovement(unitID, x, z, moveX, moveZ, speedMult, 5/jitter, 1.5, frame)
	else
		SetRabbitMovement(unitID, x, z, moveX, moveZ, speedMult, 1/jitter, speedMult^-0.1, frame)
	end
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- External functions

local function ScareRabbitsInArea(x, z, scaryAttributes)
	local frame = spGetGameFrame()
	for _, unitID in ipairs(OrderedRabbits()) do
        local data = rabbits[unitID]
		local rx,_,rz = spGetUnitPosition(unitID)
		if rx then
			local distSq = DistSq(x, z, rx, rz)
			if distSq < scaryAttributes.radiusSq then
				local magnitude = GetThingAdjustedMagnitude(sqrt(distSq), scaryAttributes)
				UpdateRabbit(unitID, frame, {x, z, magnitude})
			end
		end
	end
end

local function SetRabbitPanicResist(val)
	global_rabbitPanicResist = val
end

local function SetRabbitSpeedMult(val)
	global_rabbitSpeedMult = val
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- gadget handler functions

function gadget:UnitCreated(unitID, unitDefID)
	if spGetUnitIsDead(unitID) then
		return
	end
	
	if rabbitDefID[unitDefID] then
		AddRabbit(unitID)
	end
	if desirableUnitDefs[unitDefID] then
		AddDesirableUnit(unitID, unitDefID)
	end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if rabbitDefID[unitDefID] then
		RemoveRabbit(unitID)
	end
	if desirableUnitDefs[unitDefID] then
		RemoveDesirableUnit(unitID)
	end
end

function gadget:GameFrame(frame)
	for _, unitID in ipairs(OrderedRabbits()) do
        local data = rabbits[unitID]
		if (not data.nextUpdate) or frame >= data.nextUpdate then
			UpdateRabbit(unitID, frame)
		end
	end
end

function gadget:Initialize()
	GG.RabbitFoodDestroyed = RabbitFoodDestroyed
	GG.ScareRabbitsInArea = ScareRabbitsInArea
	GG.AddScaryArea = AddScaryArea
	GG.RemoveScaryArea = RemoveScaryArea
	GG.AddDesirableArea = AddDesirableArea
	GG.RemoveDesirableArea = RemoveDesirableArea
	
	GG.SetRabbitPanicResist = SetRabbitPanicResist
	GG.SetRabbitSpeedMult = SetRabbitSpeedMult
	
	global_rabbitSpeedMult = 1
	global_rabbitPanicResist = 1
	
	for _, unitID in ipairs(spGetAllUnits()) do
		local unitDefID = spGetUnitDefID(unitID)
		if rabbitDefID[unitDefID] then
			spSetUnitRulesParam(unitID, "internalDestroy", 1)
			spDestroyUnit(unitID, false, false)
		else
			gadget:UnitCreated(unitID, unitDefID)
		end
	end
	
	spSetGameRulesParam("rabbits_killed", 0)
end
