--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Invulnerable Units",
      desc      = "Makes some units invulnerable.",
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
local vulnerableUnitDefs = {
	[UnitDefNames["rabbit"].id] = true,
	[UnitDefNames["lighthouse"].id] = true,
	[UnitDefNames["mine"].id] = true,
}


function gadget:UnitPreDamaged(unitID, unitDefID, unitTeam, damage, paralyzer, 
                            weaponDefID, attackerID, attackerDefID, attackerTeam)
	if not vulnerableUnitDefs[unitDefID] then
		return 0
	end
	return damage
end
