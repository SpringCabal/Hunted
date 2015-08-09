function widget:GetInfo()
	return {
		name      = 'Flaoting text',
		desc      = 'Displays floating text on events',
		author    = 'gajop',
		date      = 'August 2015',
		license   = 'GNU GPL v2',
		layer     = 0,
		enabled   = true,
		handler   = true,
	}
end

local rabbitDefID = UnitDefNames["rabbit"].id

local rabbitCount = Spring.GetTeamUnitDefCount(Spring.GetMyTeamID(), rabbitDefID)
local rabbitsKilled = Spring.GetGameRulesParam("rabbits_killed") or 0
local carrotCount = Spring.GetGameRulesParam("carrot_count") or -1
local carrotsStolen = Spring.GetGameRulesParam("carrots_stolen") or 0

local events = {}
local majorEvent

local vsx, vsy

function WG.AddEvent(str, fontSize, color)
    table.insert(events, {
        str = str,
        fontSize = fontSize,
        timeout = 30,
        color = color,
    })
end

function widget:Initialize()
    vsx, vsy = Spring.GetViewGeometry()
end

function widget:DrawScreen()
    local startPos = vsy / 2 - 100
    gl.PushMatrix()
    for i = 1, #events do
        local event = events[i]
        local str, fontSize, timeout, color = event.str, event.fontSize, event.timeout, event.color
        local pos = startPos - event.timeout / 30 * 200
        local size = fontSize - math.abs(15 - event.timeout) / 30 * 12
        local fw = gl.GetTextWidth(str) * size
        gl.Color(color[1], color[2], color[3], color[4])
        gl.Text(event.str, (vsx - fw) / 2, pos, size)
    end
    gl.PopMatrix()
end
function widget:GameFrame()
    for i = #events, 1, -1 do
        local event = events[i]
        event.timeout = event.timeout - 1
        if event.timeout <= 0 then
            table.remove(events, i)
        end
    end
    
    if Spring.GetGameFrame() % 15 == 0 then
        local rabbitCountCR = Spring.GetTeamUnitDefCount(Spring.GetMyTeamID(), rabbitDefID)
        local rabbitsKilledCR = Spring.GetGameRulesParam("rabbits_killed") or 0
         if rabbitCountCR ~= rabbitCount then
            local diff = rabbitCountCR - rabbitCount
            local size = math.min(20 + math.abs(diff) * 3, 40)
            if diff > 0 then
                diff = "+" .. tostring(diff)
            else
                diff = tostring(diff)
            end
            WG.AddEvent("Rabbits: " .. diff, size, {0, 0, 1, 1})
        end
        rabbitCount = rabbitCountCR
        rabbitsKilled = rabbitsKilledCR
    elseif Spring.GetGameFrame() % 15 == 7 then
        local carrotCountCR = Spring.GetGameRulesParam("carrot_count") or -1
        local carrotsStolenCR = Spring.GetGameRulesParam("carrots_stolen") or 0
        if carrotCountCR ~= carrotCount then
            local diff = carrotCountCR - carrotCount
            if diff > 0 then
                diff = "+" .. tostring(diff)
            else
                diff = tostring(diff)
            end
            WG.AddEvent("Carrots: " .. diff, 30, {1, 0.5, 0.31, 1})
        end
        carrotCount = carrotCountCR
        carrotsStolen = carrotsStolenCR
    end
end

function widget:Update()
end