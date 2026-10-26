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

-- One frame scheduler for all burrows. This preserves family sizes and the
-- original 33 ms sleep quantization without one unit-script coroutine per wave.
local burrows, order, dirty = {}, {}, true
local RABBIT_DEF_ID = UnitDefNames.rabbit.id
local function SortedBurrows()
    if dirty then
        order = {}
        for id in pairs(burrows) do order[#order+1] = id end
        table.sort(order)
        dirty = false
    end
    return order
end
local function RandomBurrow()
    local ids = SortedBurrows()
    return burrows[ids[math.random(1, #ids)]]
end
local function SpawnWaveWithAttributes(picks, rabbitCount, familySize, familyGap, burrowFinder)
    if rabbitCount < 1 or #SortedBurrows() == 0 then return end
    local perBurrow = rabbitCount / picks
    local left, last = rabbitCount, rabbitCount
    for _ = 1, picks do
        local data = (burrowFinder or RandomBurrow)()
        left = left - perBurrow
        local count = math.ceil(last - left)
        last = math.floor(left)
        if data and burrows[data.unitID] then
            data.pending = data.pending + count
            data.nextFrame = Spring.GetGameFrame() + 1
            data.familySize, data.familyGap = familySize, familyGap
        end
    end
end
local function SpawnBurrow(x, z)
    local id = Spring.CreateUnit(BURROW_DEF_ID, x, Spring.GetGroundHeight(x, z), z, 0, 0, false, false)
    if id then Spring.SetUnitRotation(id, 0, math.random()*2*math.pi, 0) end
end
function gadget:UnitCreated(id, def)
    if def == BURROW_DEF_ID and not Spring.GetUnitIsDead(id) then
        local x,y,z = Spring.GetUnitPosition(id)
        burrows[id] = {unitID=id, x=x, y=y, z=z, pending=0}
        dirty = true
    end
end
function gadget:UnitDestroyed(id)
    if burrows[id] then burrows[id], dirty = nil, true end
end
function gadget:GameFrame(frame)
    for _, id in ipairs(SortedBurrows()) do
        local data = burrows[id]
        if data.pending > 0 and frame >= data.nextFrame then
            local family, gap = data.familySize, data.familyGap
            local count = math.min(data.pending, math.ceil(family[1] + family[2]*math.random()))
            for _ = 1, count do
                local radius = (12*math.random())^2
                local angle = math.random()*2*math.pi
                Spring.CreateUnit(RABBIT_DEF_ID, data.x + radius*math.cos(angle), data.y,
                    data.z + radius*math.sin(angle), math.random(0,3), 0, false, false)
            end
            data.pending = data.pending - count
            data.nextFrame = frame + math.ceil(0.99*(gap[1] + gap[2]*math.random()))
        end
    end
end
function gadget:Initialize()
    for _, id in ipairs(Spring.GetAllUnits()) do
        if Spring.GetUnitDefID(id) == BURROW_DEF_ID then Spring.DestroyUnit(id, false, false) end
    end
    GG.SpawnBurrow = SpawnBurrow
    GG.FindBurrows = SortedBurrows
    GG.SpawnWaveWithAttributes = SpawnWaveWithAttributes
end
