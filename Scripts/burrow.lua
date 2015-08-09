
local SIG_SPAWN = 1

local RABBIT_DEF_ID = UnitDefNames["rabbit"].id

local x,y,z
local rabbitsToSpawn = 0

function script.Create()
	x,y,z = Spring.GetUnitPosition(unitID)
end

local function RandPoint()
	local mag = (12*math.random())^2
	local dir = math.random()*2*math.pi
	
	return mag*math.cos(dir), mag*math.sin(dir)
end

local function RabbitSpawnThread(spawnCount, spawnGap)
	Signal(SIG_SPAWN)
	SetSignalMask(SIG_SPAWN)
	
	Sleep(33) -- Might be interrupted in the same frame
	
	while rabbitsToSpawn > 0 do
		local spawnCount = math.min(rabbitsToSpawn, math.ceil(spawnCount[1] + spawnCount[2]*math.random()))
		for i = 1, spawnCount do
			local xr, zr = RandPoint()
			Spring.CreateUnit(RABBIT_DEF_ID, x + xr, y, z + zr, math.random(0,3), 0, false, false)
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