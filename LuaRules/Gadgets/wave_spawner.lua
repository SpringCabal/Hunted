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
		layer	= 0,
		enabled = true
	}
end

-- constants
local GAIA_TEAM_ID = Spring.GetGaiaTeamID()
local BURROW_DEF_ID = UnitDefNames["burrow"].id
local RABBIT_DEF_ID = UnitDefNames["rabbit"].id

local burrows = {}

local function FindBurrows()
	-- assume burrows are always on gaia team
	burrows = Spring.GetTeamUnitsByDefs(GAIA_TEAM_ID, BURROW_DEF_ID)
end
GG.FindBurrows = FindBurrows

-- example burrow finder function
local function RandomBurrow()
	return burrows[math.random(1, #burrows)]
end

local function SpawnWaveWithAttributes(spawnSize, familySize, burrowFinder)
	-- spawnSize: number of rabbit families in this wave
	-- familySize: (maximum?) number of rabbits to emerge from a single burrow as a family
	-- burrowFinder: function to pass to restrict which burrows are used (TODO - purpose is to e.g. only allow burrows in the south of map, or only odd numbered etc)
	burrowFinder = burrowFinder or RandomBurrow -- default to random pick of any burrow
	for famNum = 1, spawnSize do
		for rabbitNum = 1, familySize do
			local burrowID = burrowFinder()
			local x,y,z = Spring.GetUnitPosition(burrowID)
			Spring.CreateUnit(RABBIT_DEF_ID, x,y,z, math.random(0,3), 0, false, false)
		end
	end
end
GG.SpawnWaveWithAttributes = SpawnWaveWithAttributes


-- TESTING PURPSOES ONLY for now just spawn a new wave each time a burrow is /give n
function gadget:UnitCreated(unitID, unitDefID)
	if unitDefID == BURROW_DEF_ID then
		table.insert(burrows, unitID)
		SpawnWaveWithAttributes(math.random(1, 10), math.random(1, 10), RandomBurrow)
	end
end

