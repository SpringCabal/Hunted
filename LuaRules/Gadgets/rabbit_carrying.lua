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
		layer	= -1,
		enabled = true
	}
end

local function RabbitPickupCarrot(unitID)

end

local function RabbitDropCarrot(unitID)

end

function gadget:Initialize()
	GG.RabbitPickupCarrot = RabbitPickupCarrot
	GG.RabbitDropCarrot = RabbitDropCarrot
end