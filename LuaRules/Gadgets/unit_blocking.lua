--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Blocking",
      desc      = "Makes some units non-blocking.",
      author    = "Google Frog",
      date      = "8 August 2015",
      license   = "GNU GPL, v2 or later",
      layer     = 0,
      enabled   = true
   }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
local nonBlockingUnitDefID = {
	[UnitDefNames["carrot"].id] = true,
	[UnitDefNames["burrow"].id] = true,
}


function gadget:UnitCreated(unitID, unitDefID)
	if nonBlockingUnitDefID[unitDefID] then
		Spring.SetUnitBlocking(unitID, false, false)
	end
end
