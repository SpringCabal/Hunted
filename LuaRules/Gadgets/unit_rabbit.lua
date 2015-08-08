function gadget:GetInfo()
	return {
		name = "Rabbit settings",
		desc = "Radditz",
		author = "gajop",
		date = "August 2015",
		license = "GNU GPL, v2 or later",
		layer = 1,
		enabled = true
	}
end

local rabbitDefId = UnitDefNames["rabbit"].id

if (gadgetHandler:IsSyncedCode()) then

local mines = {}

function gadget:UnitCreated(unitID, unitDefID)
    if unitDefID == rabbitDefId then
        Spring.SetUnitNoSelect(unitID, true)
        Spring.SetUnitNoMinimap(unitID, true)
    end
end

-- UNSYNCED
else

end