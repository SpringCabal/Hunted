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

local function SpawnWaveWithAttributes(burrows, rabbitsPerBurrow, familySize, familyGap, burrowFinder)
	-- A random number is a table {a, b} which results in a random number from a to a+b
	
	-- burrows: The number of burrows to pick from.
	-- rabbitsPerBurrow: The number of rabbits to spawn from each burrow.
	-- familySize: Random number of rabbits to spawn in a clump.
	-- familyGap: Random gap between spawns (in frames.
	-- burrowFinder: function to pass to restrict which burrows are used (TODO - purpose is to e.g. only allow burrows in the south of map, or only odd numbered etc)
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
		if burrowData.ScriptSpawnRabbits then
			Spring.UnitScript.CallAsUnit(burrowData.unitID, burrowData.ScriptSpawnRabbits, rabbitsPerBurrow, familySize, familyGap)
		else
			Spring.Echo("burrowData.ScriptSpawnRabbits problem")
		end
	end
end

local function CreateBurrow(x, z)

end

-- TESTING PURPSOES ONLY for now just spawn a new wave each time a burrow is /give n
function gadget:UnitCreated(unitID, unitDefID)
	if unitDefID == BURROW_DEF_ID then
		burrowData = {
			unitID = unitID,
		}
		burrows[#burrows + 1] = burrowData
	end
end

function gadget:Initialize()
	GG.CreateBurrow = CreateBurrow
	GG.FindBurrows = FindBurrows
	GG.SpawnWaveWithAttributes = SpawnWaveWithAttributes
end

