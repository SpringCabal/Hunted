--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Flare",
		desc      = "Handles flare mechanics",
		author    = "gajop",
		date      = "June, 2013",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true  --  loaded by default?
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------


local FLARE_DURATION = 6 -- in seconds
local killsRequired = 50

--SYNCED
if gadgetHandler:IsSyncedCode() then

local previousky_killed = 0
local state = "charging"
local firstActive = true
local progress = 0
local active_time = 0
local rabbits_killed

function gadget:Initialize()
    SetState("charging")
end

function SetState(newState)
    state = newState
    Spring.SetGameRulesParam("flare_state", state)
end

function SetChargingProgress(newProgress)
    progress = newProgress
    Spring.SetGameRulesParam("flare_progress", progress)
    if progress >= 100 then
        SetState("active")
        active_time = FLARE_DURATION * 30
    end
end

function gadget:GameFrame()
    if state == "charging" then
        rabbits_killed = Spring.GetGameRulesParam("rabbits_killed")
        SetChargingProgress((rabbits_killed - previousky_killed) * 100 / killsRequired)
    else
		if firstActive then
			local count = Spring.GetGameRulesParam("mine_ammo")
			Spring.SetGameRulesParam("mine_ammo", count + 5)
			firstActive = false
		end
	
        active_time = active_time - 1
        if active_time <= 0 then
            rabbits_killed = Spring.GetGameRulesParam("rabbits_killed")
            previousky_killed = rabbits_killed
            SetState("charging")
			
			firstActive = true
			killsRequired = killsRequired + 10
			
            SetChargingProgress((rabbits_killed - previousky_killed) * 100 / killsRequired)
        end
    end
end

-- UNSYNCED
else

local startedTime
-- first we have a high impulse flash and then a cooldown period
local FLASH_TIME = 0.3
local NON_FLASH_TIME = FLARE_DURATION - FLASH_TIME

function gadget:DrawScreen()
    if startedTime == nil then return end

    vsx, vsy = Spring.GetViewGeometry()
    local now = Spring.GetGameFrame()
    local alpha 
    gl.PushMatrix()
    if now - startedTime < FLASH_TIME * 30 then
        alpha = math.min(0.7, 0.2 + (now - startedTime) / 30)
    else
        alpha = math.max(0, 0.7 - (now - startedTime) / 30 / NON_FLASH_TIME)
    end
    gl.Color(1, 1, 1, alpha)
    gl.BeginEnd(GL.QUADS, function()
        gl.Vertex(0, 0)
        gl.Vertex(0, vsy)
        gl.Vertex(vsx, vsy)
        gl.Vertex(vsx, 0)
        gl.Vertex(0, 0)
    end)
    gl.PopMatrix()
end

function gadget:GameFrame()
    if Spring.GetGameRulesParam("flare_state") == "active" then
        if startedTime == nil then
            startedTime = Spring.GetGameFrame()
        end
    else
        startedTime = nil
    end
end

end