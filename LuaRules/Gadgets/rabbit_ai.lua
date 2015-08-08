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
		layer	= 0,
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
local carrotDefID = {
	[UnitDefNames["carrot"].id] = true
}
local burrowDefID = {
	[UnitDefNames["burrow"].id] = true
}

-- Rabbit fear goes down by (1 - FEAR_DECAY)*100% per frame.
-- For example 10 seconds after having 100 fear a rabbit will
-- have 100*0.995^300 = 22.2 fear
local FEAR_DECAY = 0.995

local carrotAttributes = {
	radius = 500,
	radiusSq = 500^2,
	edgeMagnitude = 50, -- Magnitude once within radius
	proximityMagnitude = 150, -- Maximum agnitude gained by being close
	thingType = 1, -- Food
}

-- Fields are manually placed groups of carrots. They need to span the whole
-- map to attract rabbits.
local desirableFieldAttributes = {
	radius = 5000,
	radiusSq = 5000^2,
	edgeMagnitude = 0, -- Magnitude once within radius
	proximityMagnitude = 80, -- Maximum agnitude gained by being close
	thingType = 1, -- Food
}

local burrowAttributes = {
	radius = 2000,
	radiusSq = 2000^2,
	edgeMagnitude = 0, -- Magnitude once within radius
	proximityMagnitude = 100, -- Maximum magnitude gained by being close
	thingType = 2, -- Safety
}

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

local carrots = {}
local burrows = {}
local rabbits = {}

-- Rabbit AI is created by taking into account only the closest scary and desirable thing.
 
-------------------------------------------------------------------
-------------------------------------------------------------------
-- 2DVector Functions

local function DistSq(x1,z1,x2,z2)
	return (x1 - x2)*(x1 - x2) + (z1 - z2)*(z1 - z2)
end

local function Mult(b, v)
	return {b*v[1], b*v[2]}
end

local function ABS(v)
	return sqrt(v[2]*v[2] + v[2]*v[2])
end

local function Unit(v)
	local mag = ABS(v)
	if mag > 0 then
		return {v[1]/mag, v[2]/mag}
	else
		return v
	end
end

local function Norm(b, v)
	local mag = ABS(v)
	if mag > 0 then
		return {b*v[1]/mag, b*v[2]/mag}
	else
		return v
	end

end

local function Add(v1, v2)
	return {v1[1] + v2[1], v1[2] + v2[2]}
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
	
	if bestMagnitude > 0 then
		return bestIndex, bestX, bestZ, bestMagnitude
	else
		return false, x, z, 0
	end
end


local function AddThing(thingTable, data)
	thingTable[#thingTable + 1] = data
end

local function RemoveThing(thingTable, index)
	thingTable[index] = thingTable[#thingTable]
	thingTable[#thingTable] = nil
end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Rabbit Handling

local function AddRabbit(unitID)
	Spring.Echo(unitID)
	rabbits[unitID] = {
		fear = 50,
		boldness = 50,
		hunger = 100,
		stamina = 100,
		fearAdd = 0.3, -- fear added every frame
		boldnessAdd = 0.5,
		lastUpdate = Spring.GetGameFrame(),
	}
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
	rabbitData.nextUpdate = frame + math.random(10, 15)
	
	--// Update Rabbit disposition.
	local scaryId, sX, sZ, scaryMag, scaryFear
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
		scaryId, sX, sZ, scaryMag = GetBestThing(scaryThings, x, z)
		scaryFear = scaryMag*updateGap
	end
	
	if rabbitData.panicMode and scaryMag + 100 > rabbitData.fear then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end
	
	rabbitData.fear = (rabbitData.fear + scaryFear + rabbitData.fearAdd*updateGap)*(FEAR_DECAY^updateGap)
	
	rabbitData.boldness = (rabbitData.boldness + rabbitData.boldnessAdd*updateGap)*(1/(0.99 + rabbitData.fear/(10000 - math.min(5000, rabbitData.boldness*15))))^updateGap
	
	local goalId, gX, gZ, goalMag = GetBestThing(desirableThings, x, z)
	
	if rabbitData.panicMode then
		if rabbitData.fear < 150 then
			rabbitData.panicMode = false
		end
	elseif rabbitData.fear > 200 then
		rabbitData.panicMode = {
			x = sX,
			z = sZ,
			mag = scaryMag,
		}
	end
	
	--// Determine move direction and speed
	-- scaryVec is the direction and magnitude to the scariest thing.
	-- goalVec is the same for the goal
	local scaryVec
	if rabbitData.panicMode then
		scaryVec = {x - rabbitData.panicMode.x, z - rabbitData.panicMode.z}
	else
		scaryVec = {x - sX, z - sZ}
	end
	
	-- These vectors are scaled by fear and boldness
	-- If fear is too high then the goal is ignored.
	-- moveVec is the resultant move direction from fear and boldness.
	scaryVec = Norm(rabbitData.fear, scaryVec)
	
	local moveVec
	if rabbitData.fear < 180 then
		local goalVec = {x - gX, z - gZ}
		goalVec = Norm(rabbitData.boldness, goalVec)
		moveVec = Add(scaryVec, goalVec)
	else
		moveVec = scaryVec
	end 
	
	-- The movement direction is now randomized a bit because rabbits are not particularly bold.
	
	local moveDir = Spring.GetHeadingFromVector(moveVec[1], moveVec[2])/2^15*math.pi
	
	--Spring.MarkerAddPoint(moveVec[1], 0 ,moveVec[2])

	--Spring.Echo("Move", moveVec[1], moveVec[2])
	--Spring.Echo("Fear", rabbitData.fear)
	--Spring.Echo("Boldness", rabbitData.boldness)
	
	local ranFocus = 2/4*math.pi
	
	local ranDir = math.random(2*math.pi)
	local ranMag = 8 +  math.random(2)
	local ranX = ranMag*math.cos(ranDir)
	local ranZ = ranMag*math.sin(ranDir)
	
	if rabbitData.panicMode then
		--GiveClampedOrderToUnit(unitID, CMD_INSERT, {0, CMD_MOVE, CMD_OPT_INTERNAL, cx,cy,cz }, {"alt"} )
		
	end
	
end

local function ScareRabbitsInArea(x, z, scaryAttributes)
	local frame = Spring.GetGameFrame()
	for unitID, data in pairs(rabbits) do
		local rx,_,rz = Spring.GetUnitPosition(unitID)
		local distSq = DistSq(x, z, rx, rz)
		if rx and distSq < scaryAttributes.radiusSq then
			local magnitude = GetThingAdjustedMagnitude(sqrt(distSq), scaryAttributes)
			Spring.Echo("Updating")
			UpdateRabbit(unitID, frame, {x, z, magnitude})
		end
	end
end


-------------------------------------------------------------------
-------------------------------------------------------------------
-- External Functions

function gadget:UnitCreated(unitID, unitDefID)
	if rabbitDefID[unitDefID] then
		AddRabbit(unitID)
	end
	if carrotDefID[unitDefID] then
		carrots[unitID] = true
	end
	if burrowDefID[unitDefID] then
		burrows[unitID] = true
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
	GG.ScareRabbitsInArea = ScareRabbitsInArea
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end