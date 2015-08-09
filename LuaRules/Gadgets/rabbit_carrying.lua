if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Rabbit Carrying",
		desc	= "Implements picking up, carrying and dropping carrots for rabbits",
		author	= "Google Frog",
		date	= "9 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 5,
		enabled = true
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local carriedDefID = UnitDefNames["carrot_carried"].id

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local rabbitCarrying = {}
local carrriedCarrotRabbit = {}

local function RabbitPickupCarrot(unitID)
	local _,_,_,x,y,z = Spring.GetUnitPosition(unitID, true)
	
	local carryID = Spring.CreateUnit(carriedDefID, x, y, z, 0, 0, false, false)

	carrriedCarrotRabbit[carryID] = unitID
	rabbitCarrying[unitID] = carryID
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	if env and env.AttachCarrot then
		Spring.UnitScript.CallAsUnit(unitID, env.AttachCarrot, carryID)
	end
end

local function RabbitDropCarrot(unitID)
	local carryID = rabbitCarrying[unitID]
	if carryID and Spring.ValidUnitID(carryID) then
		local vx, vy, vz = Spring.GetUnitVelocity(unitID)
		
		local env = Spring.UnitScript.GetScriptEnv(unitID)
		if env and env.DropCarrot then
			Spring.UnitScript.CallAsUnit(unitID, env.DropCarrot, carryID)
		end
		
		Spring.SetUnitBlocking(unitID, false, false)
		
		local carEnv = Spring.UnitScript.GetScriptEnv(carryID)
		if carEnv and carEnv.BeDropped then
			Spring.UnitScript.CallAsUnit(carryID, carEnv.BeDropped)
		end
		
		local speedSq = vx*vx + vy*vy + vz*vz
		if speedSq > 50 then
			local speed = math.sqrt(speedSq)
			vx, vy, vz = vx/speed, vy/speed, vz/speed
		end
		
		Spring.SetUnitVelocity(carryID, vx, vy, vz)
	end
end

local function RabbitScoreCarrot(unitID)
	local carryID = rabbitCarrying[unitID]
	if carryID and Spring.ValidUnitID(carryID) then
		Spring.SetUnitRulesParam(carryID, "carrotScored", 1)
		Spring.DestroyUnit(carryID, false, false)
	end
	rabbitCarrying[unitID] = nil
	
	local stolen = Spring.GetGameRulesParam("carrots_stolen")
	Spring.SetGameRulesParam("carrots_stolen", stolen + 1)
	--Spring.Echo("A Rabbit return a Carrot to their Burrow!!")
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if carriedDefID ~= unitDefID then 
		return
	end
	if Spring.GetUnitRulesParam(unitID, "internalDestroy") then
		carrriedCarrotRabbit[unitID] = nil
		return
	end
	
	local carrotCount = Spring.GetGameRulesParam("carrot_count") or 0
	Spring.SetGameRulesParam("carrot_count", carrotCount - 1)
	
	if not Spring.GetUnitRulesParam(unitID, "carrotScored") then
		local destroyed = Spring.GetGameRulesParam("carrots_destroyed")
		Spring.SetGameRulesParam("carrots_destroyed", destroyed + 1)
	end
	
	if GG.RabbitFoodDestroyed then
		GG.RabbitFoodDestroyed(carrriedCarrotRabbit[unitID])
	end
	
	carrriedCarrotRabbit[unitID] = nil
end

function gadget:Initialize()
	GG.RabbitPickupCarrot = RabbitPickupCarrot
	GG.RabbitDropCarrot = RabbitDropCarrot
	GG.RabbitScoreCarrot = RabbitScoreCarrot
	
	-- Clean up carried carrots as rabbits will not remember that they exist.
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if unitDefID == carriedDefID then
			Spring.DestroyUnit(unitID, false, false)
		end
	end
end