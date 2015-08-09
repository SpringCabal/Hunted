
local SIG_SPAWN = 1

local RABBIT_DEF_ID = UnitDefNames["rabbit"].id

local x,y,z
local rabbitsToSpawn = 0

function script.Create()
	x,y,z = Spring.GetUnitPosition(unitID)
end

local function RabbitSpawnThread(spawnCount, spawnGap)
	Signal(SIG_SPAWN)
	SetSignalMask(SIG_SPAWN)
	
	while rabbitsToSpawn > 0 do
		local spawnCount = math.min(rabbitsToSpawn, math.ceil(spawnCount[1] + spawnCount[2]*math.random()))
		for i = 1, spawnCount do
			Spring.CreateUnit(RABBIT_DEF_ID, x,y,z, math.random(0,3), 0, false, false)
		end
	
		rabbitsToSpawn = rabbitsToSpawn - spawnCount
		Sleep(33 * (spawnGap[1] + spawnGap[2]*math.random()))
	end
end

function SpawnRabbits(totalCount, spawnCount, spawnGap)
	rabbitsToSpawn = rabbitsToSpawn + totalCount
	StartThread(RabbitSpawnThread, spawnCount, spawnGap)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end