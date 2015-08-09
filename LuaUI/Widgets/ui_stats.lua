function widget:GetInfo()
	return {
		name      = 'Stats',
		desc      = 'Display wave information',
		author    = 'gajop',
		date      = 'August 2015',
		license   = 'GNU GPL v2',
		layer     = 0,
		enabled   = true,
		handler   = true,
	}
end

local carrotDefID = UnitDefNames["carrot"].id
local rabbitDefID = UnitDefNames["rabbit"].id

local lblRabbits, lblCarrots

function widget:Initialize()
	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end

	Chili = WG.Chili
	screen0 = Chili.Screen0
	
    lblRabbits = Chili.Label:New {
        right = 10,
        width = 100,
        y = 10,
        height = 40,
        parent = screen0,
        font = {
            size = 24,
        }
    }
    lblCarrots = Chili.Label:New {
        right = 10,
        width = 100,
        y = 45,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        }
    }
    UpdateRabbits()
    UpdateCarrots()
end

function widget:UpdateRabbits()
    local rabbitCount = Spring.GetTeamUnitDefCount(Spring.GetMyTeamID(), rabbitDefID)
    lblRabbits:SetCaption("\255\30\144\255Rabbits: " .. rabbitCount .. "\b")
end

function widget:UpdateCarrots()
    local carrotCount = Spring.GetGameRulesParam("carrot_count") or -1
    lblCarrots:SetCaption("\255\255\165\0Carrots: " .. carrotCount .. "\b")
end

function widget:GameFrame()
    UpdateRabbits()
    UpdateCarrots()
end
-- function widget:UnitDestroyed(unitID, unitDefID, ...)
--     Spring.Echo("unit created")
--     if unitDefID == rabbitDefID then
--         UpdateRabbits()
--     elseif unitDefID == carrotDefID then
--         UpdateCarrots()
--     end
-- end
-- 
-- function widget:UnitCreated(unitID, unitDefID, ...)
--     Spring.Echo("unit created")
--     if unitDefID == rabbitDefID then
--         UpdateRabbits()
--     elseif unitDefID == carrotDefID then
--         UpdateCarrots()
--     end
-- end