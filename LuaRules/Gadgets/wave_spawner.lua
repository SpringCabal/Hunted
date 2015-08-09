if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Rabbit Spawner",
		desc	= "Implements Rabbits spawning from burrows",
		author	= "FLOZi",
		date	= "8 August 2015",
		license	= "GNU GPL, v2 or later",
		layer	= 15,
		enabled = true
	}
end

-- constants
local GAIA_TEAM_ID = Spring.GetGaiaTeamID()
local BURROW_DEF_ID = UnitDefNames["burrow"].id

local burrows = {}

local function FindBurrows()
	-- assume burrows are always on gaia team
	burrows = Spring.GetTeamUnitsByDefs(GAIA_TEAM_ID, BURROW_DEF_ID)
end

-- example burrow finder function
local function RandomBurrow()
	return burrows[math.random(1, #burrows)]
end

local function SpawnWaveWithAttributes(burrows, rabbitCount, familySize, familyGap, burrowFinder)
	-- A random number is a table {a, b} which results in a random number from a to a+b
	
	if rabbitCount < 1 then
		return
	end
	
	-- burrows: The number of burrows to pick from.
	-- rabbitCount: The number of rabbits to spawn in total
	-- familySize: Random number of rabbits to spawn in a clump.
	-- familyGap: Random gap between spawns (in frames.
	-- burrowFinder: function to pass to restrict which burrows are used (TODO - purpose is to e.g. only allow burrows in the south of map, or only odd numbered etc)
	local rabbitsPerBurrow = rabbitCount/burrows
	local rabbitsLeft = rabbitCount
	local lastRabbits = rabbitCount
	
	
	burrowFinder = burrowFinder or RandomBurrow -- default to random pick of any burrow
	for burrowNum = 1, burrows do
		local burrowData = burrowFinder()
		if not burrowData.ScriptSpawnRabbits then
			local env = Spring.UnitScript.GetScriptEnv(burrowData.unitID)
			if env and env.SpawnRabbits then
				burrowData.env = env
				burrowData.ScriptSpawnRabbits = env.SpawnRabbits
			end
		end
		rabbitsLeft = rabbitsLeft - rabbitsPerBurrow
		local thisBurrowRabbits = math.ceil(lastRabbits - rabbitsLeft)
		lastRabbits = math.floor(rabbitsLeft)
		
		if burrowData.ScriptSpawnRabbits then
			Spring.UnitScript.CallAsUnit(burrowData.unitID, burrowData.ScriptSpawnRabbits, thisBurrowRabbits, familySize, familyGap)
		else
			Spring.Echo("burrowData.ScriptSpawnRabbits problem")
		end
	end
end

local function SpawnBurrow(x, z)
	local burrowID = Spring.CreateUnit(BURROW_DEF_ID, x, 0, z, 0, 0, false, false)
	Spring.SetUnitRotation(burrowID, 0, math.random()*2*math.pi, 0)
end

-- TESTING PURPSOES ONLY for now just spawn a new wave each time a burrow is /give n
function gadget:UnitCreated(unitID, unitDefID)
	if Spring.GetUnitIsDead(unitID) then
		return
	end
	
	if BURROW_DEF_ID == unitDefID then
		burrowData = {
			unitID = unitID,
		}
		burrows[#burrows + 1] = burrowData
	end
end

function gadget:Initialize()
	burrows = {}
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		if BURROW_DEF_ID == unitDefID then
			Spring.DestroyUnit(unitID, false, false)
		end
	end

	GG.SpawnBurrow = SpawnBurrow
	GG.FindBurrows = FindBurrows
	GG.SpawnWaveWithAttributes = SpawnWaveWithAttributes
end

