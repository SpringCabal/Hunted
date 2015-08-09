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
		layer	= 1,
		enabled = true
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local carryDefID = UnitDefNames["carrot_carried"].id
local dropDefID = UnitDefNames["carrot_dropped"].id

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local rabbitCarrying = {}

local function RabbitPickupCarrot(unitID)
	local _,_,_,x,y,z = Spring.GetUnitPosition(unitID, true)
	
	local carryID = Spring.CreateUnit(carryDefID, x, y, z, 0, 0, false, false)

	rabbitCarrying[unitID] = carryID
	
	local env = Spring.UnitScript.GetScriptEnv(unitID)
	if env and env.AttachCarrot then
		Spring.UnitScript.CallAsUnit(unitID, env.AttachCarrot, carryID)
	end
end

local function RabbitDropCarrot(unitID)
	local carryID = rabbitCarrying[unitID]
	
	if carryID then
		Spring.DestroyUnit(carryID, false, false)
		local _,_,_,x,y,z = Spring.GetUnitPosition(unitID, true)
		Spring.CreateUnit(dropDefID, x, y, z, 0, 0, false, false)

		rabbitCarrying[unitID] = nil
	end
end

local function RabbitScoreCarrot(unitID)
	local carryID = rabbitCarrying[unitID]
	if carryID then
		Spring.DestroyUnit(carryID, false, false)
		rabbitCarrying[unitID] = nil
	end
	Spring.Echo("A Rabbit return a Carrot to their Burrow!!")
end

function gadget:Initialize()
	GG.RabbitPickupCarrot = RabbitPickupCarrot
	GG.RabbitDropCarrot = RabbitDropCarrot
	GG.RabbitScoreCarrot = RabbitScoreCarrot
	
	-- Clean up carried carrots as rabbits will not remember that they exist.
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if unitDefID == carryDefID then
			Spring.DestroyUnit(unitID, false, false)
		end
	end
end