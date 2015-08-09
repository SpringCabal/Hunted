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
local lighthouseDefID = UnitDefNames["lighthouse"].id
local mineDefID = UnitDefNames["mine"].id

local startFrame = 0
local waveGap = 550
local MAX_RABBITS = 600

local currentDifficult = 1

local waveAtt = {
	burrows = 5,
	rabbitCount = 25,
	familySize = {2, 2},
	familyGap = {12, 10},
}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local function SpawnUnit(unitDefID, x, z, noRotate)
	local unitID = Spring.CreateUnit(unitDefID, x, 0, z, 0, 0, false, false)
	if not noRotate then
		Spring.SetUnitRotation(unitID, 0, math.random()*2*math.pi, 0)
	end
end

local function CleanUnits()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if lighthouseDefID == unitDefID or unitDefID == mineDefID then
			Spring.SetUnitRulesParam(unitID, "internalDestroy", 1)
			Spring.DestroyUnit(unitID, false, false)
		end
	end
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:Initialize()
	CleanUnits()
	
	GG.SpawnField(2650, 2125, 2930, 2380, 4, 5)
	GG.SpawnField(2320, 3050, 2505, 3550, 8, 3)
	GG.SpawnField(3230, 3900, 3890, 4190, 4, 11)
	GG.SpawnField(3745, 2360, 3870, 2570, 4, 3)

	GG.SpawnBurrow(3600, 1350)
	GG.SpawnBurrow(4740, 1940)
	GG.SpawnBurrow(4650, 3340)
	GG.SpawnBurrow(4920, 4620)
	GG.SpawnBurrow(3680, 5270)
	GG.SpawnBurrow(2190, 4820)
	GG.SpawnBurrow(1000, 3660)
	GG.SpawnBurrow( 800, 2170)
	GG.SpawnBurrow(2050, 1250)
	
	SpawnUnit(lighthouseDefID, 2560, 2500, true)
	SpawnUnit(lighthouseDefID, 3130, 4320, true)
	SpawnUnit(lighthouseDefID, 3980, 3810, true)
	SpawnUnit(lighthouseDefID, 3630, 2670, true)
	SpawnUnit(lighthouseDefID, 2200, 3650, true)
	
	startFrame = Spring.GetGameFrame()
	
	Spring.SetGameRulesParam("score", 0)
	Spring.SetGameRulesParam("survivalTime", 0)
	currentDifficult = 1
end

function gadget:GameFrame(frame)
	local elapsed = frame - startFrame
	if elapsed%waveGap == 5 then
		
		GG.SetRabbitPanicResist(currentDifficult)
		GG.SetRabbitSpeedMult(1 + currentDifficult/2)
		
		currentDifficult = currentDifficult * (1 + 0.04/currentDifficult)
		
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
			waveAtt.familySize[1],
			waveAtt.familySize[2]
		}
	end
	
	--// Scoring and game information
	if elapsed%30 == 29 then
		-- Update ammo at this point
		-- The UI looks better if all the numbers update at the same time
		GG.UpdateAmmo()
		
		Spring.SetGameRulesParam("survivalTime", math.ceil(elapsed/30))
		local carrots = Spring.GetGameRulesParam("carrot_count") or 0
		local score = Spring.GetGameRulesParam("score") or 0
		Spring.SetGameRulesParam("score", score + carrots)
	end
end