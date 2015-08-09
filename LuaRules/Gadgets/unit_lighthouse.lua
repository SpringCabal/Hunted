if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Lighthouse",
		desc	= "Handles Lighthouses",
		author	= "Google Frog",
		date	= "9 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 10,
		enabled = true
	}
end

-------------------------------------------------------------------
-------------------------------------------------------------------

local lighthouseDefIDs = {
	[UnitDefNames["lighthouse"].id] = {
		
	}
}

-------------------------------------------------------------------
-------------------------------------------------------------------
-- Unit Handling

local function AddLighthouse(unitID, unitDefID)


end

-------------------------------------------------------------------
-------------------------------------------------------------------
-- gadget handler functions

function gadget:UnitCreated(unitID, unitDefID)
	if Spring.GetUnitIsDead(unitID) then
		return
	end
	
	if lighthouseDefIDs[unitDefID] then
		AddLighthouse(unitID, unitDefID)
	end
end

function gadget:UnitDestroyed(unitID, unitDefID)
	if lighthouseDefIDs[unitDefID] then
		RemoveLighthouse(unitID)
	end
end