function gadget:GetInfo()
	return {
		name = "Proximity lamndines",
		desc = "Carrots.. that.. explode!",
		author = "gajop",
		date = "August 2015",
		license = "GNU GPL, v2 or later",
		layer = 1,
		enabled = true
	}
end

local LOG_SECTION = "mine"
local LOG_LEVEL = LOG.DEBUG
local triggerRadius = 80
local explosionRadius = 100
local mineDefID = UnitDefNames["mine"].id
local rabbitDefId = UnitDefNames["rabbit"].id

local scareAttributes = {
	radius = 250,
	radiusSq = 250^2,
	edgeMagnitude = 50,
	proximityMagnitude = 500,
}


if (gadgetHandler:IsSyncedCode()) then

local mines = {}

function gadget:GameFrame()
    for mineID, mineObj in pairs(mines) do
        local x, y, z = Spring.GetUnitPosition(mineID)
        -- check if any mines should trigger
        if not mineObj.exploding then
            local nearbyUnits = Spring.GetUnitsInCylinder(x, z, triggerRadius)
            for _, nearbyUnitID in pairs(nearbyUnits) do
                if Spring.GetUnitDefID(nearbyUnitID) == rabbitDefId then
                    mineObj.exploding = true
                    mineObj.explodingTimer = 3 * 30 -- 3 seconds exploding timer
                    break
                end
            end
        -- check for any mines that should explode
        else
            if mineObj.explodingTimer <= 0 then
                Spring.DestroyUnit(mineID)
            else
                SendToUnsynced("UpdateMineTimer", mineID, mineObj.explodingTimer) 
                mineObj.explodingTimer = mineObj.explodingTimer - 1
            end
        end
    end
end

function gadget:UnitCreated(unitID, unitDefID)
    if unitDefID == mineDefID then
        mines[unitID] = { exploding = false }
    end
end

function gadget:UnitDestroyed(unitID, unitDefID, unitTeam, attackerID, attackerDefID, attackerTeam)
	if mines[unitID] then
		SendToUnsynced("RemoveMineTimer", unitID) 
		local x, y, z = Spring.GetUnitPosition(unitID)
		GG.ScareRabbitsInArea(x, z, scareAttributes)
		mines[unitID] = nil
	end
end

function gadget:Initialize()
    -- handle luarules reload
    local currentFrame = Spring.GetGameFrame()
	for _, unitID in pairs(Spring.GetAllUnits()) do
		gadget:UnitCreated(unitID, Spring.GetUnitDefID(unitID), Spring.GetUnitTeam(unitID))
	end
end

-- UNSYNCED
else
--

local mineTimers = {}

function gadget:Initialize()
    gadgetHandler:AddSyncAction("UpdateMineTimer", UpdateMineTimer)
    gadgetHandler:AddSyncAction("RemoveMineTimer", RemoveMineTimer)
end

function UpdateMineTimer(_, unitID, frames)
    mineTimers[unitID] = frames
end

function RemoveMineTimer(_, unitID)
    mineTimers[unitID] = nil
end

function gadget:DrawWorld()
    for mineID, mineTimer in pairs(mineTimers) do
        local x, y, z = Spring.GetUnitPosition(mineID)
        if x ~= nil then
            local fontSize = 38
            local time = math.ceil(mineTimer / 30)
            gl.PushMatrix()
            gl.Translate(x-38/2, y+20, z-38/2)
    --        gl.Rotate(90, 1, 0, 0)
            local txt = tostring(time)
            gl.Color(1, 0, 0, 1)
            --gl.Rotate(180, 0, 0, 1)
            --gl.Scale(-1, 1, 1)
            gl.Text(txt, 0, 0, fontSize)
            gl.PopMatrix()
        end
    end
end

end