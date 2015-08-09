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
		eatTime = 80,
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
		radius = 2000,
		radiusSq = 2000^2,
		edgeMagnitude = 0.1, -- Magnitude once within radius (per frame)
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
local FEAR_ADDED = 0.3 -- fear added every frame
local BOLDNESS_ADDED = 0.3 -- boldness added every frame

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

-- Rabbit AI is created by taking into account only the closest scary and desirable thing.

-------------------------------------------------------------------
-------------------------------------------------------------------
-- 2D Vector Functions

local function DistSq(x1,z1,x2,z2)
	return (x1 - x2)*(x1 - x2) + (z1 - z2)*(z1 - z2)
end

local function Mult(b, v)
	return {b*v[1], b*v[2]}
end

local function AbsVal(v)
	return sqrt(v[1]*v[1] + v[2]*v[2])
end

local function Unit(v)
	local mag = AbsVal(v)
	if mag > 0 then
		return {v[1]/mag, v[2]/mag}
	else
		return v
	end
end

local function Norm(b, v)
	local mag = AbsVal(v)
	if mag > 0 then
		return {b*v[1]/mag, b*v[2]/mag}
	else
		return v
	end
end

local function Angle(v)
	return -Spring.GetHeadingFromVector(v[1], v[2])/2^15*math.pi + math.pi/2
end

local function PolarToCart(mag, dir)
	return {mag*math.cos(dir), mag*math.sin(dir)}
end

local function Add(v1, v2)
	return {v1[1] + v2[1], v1[2] + v2[2]}
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
	Spring.SetUnitMoveGoal(unitID, x, params[2], z, 16, nil, true) -- The last argument is whether the goal is raw
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
	
	local thing, distanceSquare, distance, thingAtt, adjustedMagnitude
	
	for i = 1, #thingTable do
		thing = thingTable[i]
		if not thing.inactive then
			thingAtt = thing.attributes
			distanceSquare = DistSq(x, z, thing.x, thing.z)
			if distanceSquare < thingAtt.radiusSq then
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
	return desirableThings[index]
end

local function RemoveDesirableArea(data)
	RemoveThing(desirableThings, data.index)
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Desirable Unit Handling

local function StartStealing(rabbitData, thingRef)
	if Spring.ValidUnitID(unitID) then
		Spring.SetUnitRulesParam(unitID, "stealing", 1)
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
	if Spring.ValidUnitID(unitID) then
		Spring.SetUnitRulesParam(unitID, "stealing", 0)
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
	local x,_,z = Spring.GetUnitPosition(unitID)
	local attributes = desirableUnitDefs[unitDefID]
	local index = AddThing(desirableThings, {
		x = x,
		z = z,
		unitID = unitID,
		attributes = attributes,
	})
	
	desirableUnits[unitID] = {thingTableEntry = desirableThings[index]}
end

local function RemoveDesirableUnit(unitID)
	if desirableUnits[unitID].eater then
		StopStealing(desirableUnits[unitID].eater)
	end

	local index = desirableUnits[unitID].thingTableEntry.index
	RemoveThing(desirableThings, index)
	desirableUnits[unitID] = nil
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Rabbit Handling

local function AddRabbit(unitID)
	rabbits[unitID] = {
		unitID = unitID,
		fear = 50,
		boldness = 150,
		stamina = 100,
		foodCarried = 0,
		lastUpdate = Spring.GetGameFrame(),
	}
end

local function RemoveRabbit(unitID)
	if rabbits[unitID].foodCarried > 0 then
		GG.RabbitDropCarrot(unitID)
	end
	
	if rabbits[unitID].eatingProgress then
		StopStealing(rabbits[unitID])
	end
	rabbits[unitID] = nil
	
	if not Spring.GetUnitRulesParam(unitID, "internalDestroy") then
		local killed = Spring.GetGameRulesParam("rabbits_killed")
		Spring.SetGameRulesParam("rabbits_killed", killed + 1)
	end
end

local function RabbitFoodDestroyed(unitID)
	local rabbitData = rabbits[unitID]
	if rabbitData and rabbitData.foodCarried then
		rabbitData.foodCarried = 0
	end
end

local function SetRabbitMovement(unitID, x, z, goalVec, speedMult, accelMult, turnMult)
	--Spring.MarkerAddPoint(goalVec[1] + x,0,goalVec[2] + z)
	Spring.SetUnitRulesParam(unitID, "selfMoveSpeedChange", speedMult)
		Spring.SetUnitRulesParam(unitID, "selfMaxAccelerationChange", accelMult)
	Spring.SetUnitRulesParam(unitID, "selfTurnSpeedChange", turnMult)
	GG.UpdateUnitAttributes(unitID)
	GiveClampedOrderToUnit(unitID, CMD.MOVE, {goalVec[1] + x, 0, goalVec[2] + z}, 0 )
end

local function UpdateRabbit(unitID, frame, scaryOverride)

	if not (Spring.ValidUnitID(unitID) and rabbits[unitID]) then
		return
	end
	
	--// Collect rabbit information
	frame = frame or Spring.GetGameFrame()
	local x,_,z = Spring.GetUnitPosition(unitID)
	local rabbitData = rabbits[unitID]

	local updateGap = frame - rabbitData.lastUpdate
	rabbitData.lastUpdate = frame
	rabbitData.nextUpdate = frame + 10 + 20*math.random()
	
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
		scaryRef, sX, sZ, scaryMag = GetBestThing(scaryThings, x, z)
		scaryFear = scaryMag*updateGap
	end
	
	-- Panic mode causes a rabbit to run away from a scary location until it calms down.
	if rabbitData.panicMode and (scaryMag + 100 > rabbitData.fear or scaryMag > 100)then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end

	rabbitData.fear = (rabbitData.fear + scaryFear + FEAR_ADDED*updateGap)*(FEAR_DECAY^updateGap)
	
	if rabbitData.panicMode then
		if rabbitData.fear < 150 then
			rabbitData.panicMode = false
		end
	elseif rabbitData.fear > 250 then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end
	
	-- Paniced Rabbits Drop Carrots
	if rabbitData.fear > 500 and rabbitData.foodCarried > 0 then
		rabbitData.foodCarried = 0
		GG.RabbitDropCarrot(unitID)
	end
	
	--// Update Goal and Boldness
	local expectedFood = rabbitData.foodCarried + ((rabbitData.eatingProgress and 1) or 0)
	-- 1 carrot
	local deisrableTypeTable = {
		math.max(0, 1 - expectedFood), -- Desirability of food
		math.min(1, expectedFood), -- Desirability of Burrow
	}
	
	local goalRef, gX, gZ, goalMag = GetBestThing(desirableThings, x, z, deisrableTypeTable)
	
	rabbitData.boldness = (rabbitData.boldness + goalMag*updateGap + BOLDNESS_ADDED*updateGap)*
		(1/(0.99 + rabbitData.fear/(10000 - math.min(8000, rabbitData.boldness*20))))^updateGap

	--// Update Speed and Stamina
	local speedMult = ((((rabbitData.fear + math.max(0, rabbitData.boldness - 320))/45)^0.8)*(2 + (rabbitData.boldness/200)^0.45)/2.3)*rabbitData.stamina/150

	-- High scaryMag means that something scary is happening right now!
	-- (Suddden blast or near center of a torch)
	if scaryMag > 0.9 then
		speedMult = speedMult*2
	end
	
	rabbitData.stamina = (rabbitData.stamina - ((speedMult)^0.2)*updateGap + 1.3*updateGap)*STAMINA_DECAY^updateGap
	
	--// Handle Carrot Eating
	-- Eat until the carrot is gone. If paniced stop eating and run away.
	if rabbitData.eatingProgress then
		if rabbitData.panicMode then
			StopStealing(rabbitData)
		else
			rabbitData.eatingProgress = rabbitData.eatingProgress + updateGap
			if rabbitData.eatingProgress > rabbitData.eatingThingRef.attributes.eatTime then
				GG.RabbitPickupCarrot(unitID)
				--Spring.Echo("Destroying", rabbitData.eatingThingRef.unitID, Spring.ValidUnitID(rabbitData.eatingThingRef.unitID))
				Spring.SetUnitRulesParam(rabbitData.eatingThingRef.unitID, "internalDestroy", 1)
				Spring.DestroyUnit(rabbitData.eatingThingRef.unitID, false, false)
				rabbitData.foodCarried = rabbitData.foodCarried + 1
				rabbitData.boldness = rabbitData.boldness + 150 -- Bonus boldness for being a good theif!
				StopStealing(rabbitData)
			else
				SetRabbitMovement(unitID, x, z, {rabbitData.eatingThingRef.x - x, rabbitData.eatingThingRef.z - z}, 0.05, 2, 0.2)
				return
			end
		end
	end
	
	--speedMult = math.min(100, speedMult^5)
	
	--GG.UnitEcho(unitID, math.floor(rabbitData.fear))
	--Spring.Echo("Fear", rabbitData.fear)
	--Spring.Echo("Boldness", rabbitData.boldness)
	--Spring.Echo("Stamina", rabbitData.stamina)
	--Spring.Echo("speedMult", speedMult)
	
	--// Determine move direction and speed
	-- scaryVec is the direction and magnitude to the scariest thing.
	-- goalVec is the same for the goal
	local scaryVec
	if rabbitData.panicMode then
		scaryVec = {rabbitData.panicMode.x - x, rabbitData.panicMode.z - z}
	else
		scaryVec = {sX - x, sZ - z}
	end
	
	-- These vectors are scaled by fear and boldness
	-- If fear is too high then the goal is ignored.
	-- moveVec is the resultant move direction from fear and boldness.
	scaryVec = Norm(-rabbitData.fear/8, scaryVec)
	
	local moveVec, goalMag
	if rabbitData.fear < 180 then
		local goalVec = {gX - x, gZ - z}
		goalMag = AbsVal(goalVec)
		if goalMag < 200 then
			speedMult = speedMult*1.2
			goalVec = Norm(rabbitData.boldness/10, goalVec)
		else
			goalVec = Norm(rabbitData.boldness/30, goalVec)
		end
		moveVec = Add(scaryVec, goalVec)
	else
		moveVec = scaryVec
	end 
	
	--// Handle Transition into Non-Moving (eating, depositing etc..)
	-- Pick up a nearby carrot
	if goalRef and goalMag and goalMag < 60 and goalRef.attributes.isEdible and 
			(not rabbitData.eatingProgress) and rabbitData.foodCarried == 0 and (not rabbitData.panicMode) then
		StartStealing(rabbitData, goalRef)
		SetRabbitMovement(unitID, x, z, {gX - x, gZ - z}, 0.05, 2, 0.2)
		return
	end
	
	-- Deposit a carrot in a burrow
	if goalRef and goalMag and goalMag < 60 and goalRef.attributes.isCarrotRepository and 
			rabbitData.foodCarried > 0 and (not rabbitData.panicMode) then
		rabbitData.foodCarried = 0
		GG.RabbitScoreCarrot(unitID)
		SetRabbitMovement(unitID, x, z, {gX - x, gZ - z}, 0.05, 2, 1)
		return
	end
	
	--// Normal Movement
	-- At this point moveVec is the direction which a rabbit would go if it were bold.
	-- Direction is 0 in positive x direction. Increases clockwise.
	local moveDir = Angle(moveVec)
	local moveMag = AbsVal(moveVec)
	
	--Spring.MarkerAddPoint(moveVec[1], 0 ,moveVec[2])
	--Spring.Echo("Move", moveVec[1], moveVec[2], moveDir)
	
	-- Boldness decreases rabbit movement jitter.
	local dirRandomness = math.min(math.pi*0.4, 20*(10 + rabbitData.boldness)^(-0.9))
	local moveDir = moveDir + math.random()*2*dirRandomness - dirRandomness
	
	local jitter = 1/(1 + math.max(0, rabbitData.boldness - 300)/100)
	
	-- Rabbits have a small bias towards keeping their momentum
	local vx, _, vz, velMag = Spring.GetUnitVelocity(unitID)
	local velVector = Norm((10 + math.random()*5)*jitter, {vx, vz})
	moveVec = PolarToCart(moveMag, moveDir)
	
	-- Rabbits also just move randomly.
	local randVec = PolarToCart((5 + math.random()*10)*jitter, math.random()*2*math.pi)
	
	--Spring.Echo("jitter", jitter)
	--Spring.Echo("moveVec Angle", Angle(moveVec)*180/math.pi)
	--Spring.Echo("randVec", AbsVal(randVec))
	--Spring.Echo("dirRandomness", dirRandomness*180/math.pi)
	
	-- Add the randomVector, momentumVector and goal+scary vectors to get final direction.
	moveVec = Norm(200*speedMult, Add(moveVec, Add(randVec, velVector)))
	
	--// Modify movement attributes and goal
	if scaryMag > 100 then
		SetRabbitMovement(unitID, x, z, moveVec, speedMult, 5/jitter, 1.5)
	else
		SetRabbitMovement(unitID, x, z, moveVec, speedMult, 1/jitter, speedMult^-0.1)
	end
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- External functions

local function ScareRabbitsInArea(x, z, scaryAttributes)
	local frame = Spring.GetGameFrame()
	for unitID, data in pairs(rabbits) do
		local rx,_,rz = Spring.GetUnitPosition(unitID)
		if rx then
			local distSq = DistSq(x, z, rx, rz)
			if distSq < scaryAttributes.radiusSq then
				local magnitude = GetThingAdjustedMagnitude(sqrt(distSq), scaryAttributes)
				UpdateRabbit(unitID, frame, {x, z, magnitude})
			end
		end
	end
end


-------------------------------------------------------------------
-------------------------------------------------------------------
-- gadget handler functions

function gadget:UnitCreated(unitID, unitDefID)
	if Spring.GetUnitIsDead(unitID) then
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
	for unitID, data in pairs(rabbits) do
		if (not data.nextUpdate) or frame >= data.nextUpdate then
			UpdateRabbit(unitID)
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
	
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if rabbitDefID[unitDefID] then
			Spring.SetUnitRulesParam(unitID, "internalDestroy", 1)
			Spring.DestroyUnit(unitID, false, false)
		else
			gadget:UnitCreated(unitID, unitDefID)
		end
	end
	
	Spring.SetGameRulesParam("rabbits_killed", 0)
end