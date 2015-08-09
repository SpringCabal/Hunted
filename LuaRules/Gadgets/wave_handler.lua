--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Wave Handler",
      desc      = "Handles the beginning and end of waves.",
      author    = "Google Frog",
      date      = "9 August 2015",
      license   = "GNU GPL, v2 or later",
      layer     = 20,
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

local rabbitDefID = UnitDefNames["rabbit"].id

local waveGap = 600
local MAX_RABBITS = 800

local waveAtt = {
	burrows = 5,
	rabbitCount = 25,
	familySize = {2, 3},
	familyGap = {8, 10},
}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------


function gadget:Initialize()
	GG.SpawnField(1000, 1800, 1600, 2200, 10, 5)
	GG.SpawnField(2200, 2100, 2700, 2350, 8, 4)
	
	GG.SpawnBurrow(600, 850)
	GG.SpawnBurrow(2500, 1060)
	GG.SpawnBurrow(3700, 2200)
	GG.SpawnBurrow(3300, 3100)
	GG.SpawnBurrow(1170, 3400)
end

function gadget:GameFrame(frame)
	if frame%waveGap == 0 then
		
		local rabbitCount = Spring.GetTeamUnitDefCount(0, rabbitDefID)
		
		GG.SpawnWaveWithAttributes(
			waveAtt.burrows,
			math.min(MAX_RABBITS - rabbitCount, waveAtt.rabbitCount),
			waveAtt.familySize,
			waveAtt.familyGap
		)
		waveAtt.burrows = math.ceil(waveAtt.burrows*1.1)
		waveAtt.rabbitCount = math.ceil(waveAtt.rabbitCount*1.1)
		waveAtt.familySize = {
			1.05*waveAtt.familySize[1],
			1.05*waveAtt.familySize[2]
		}
	end
end