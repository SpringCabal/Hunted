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

local lblRabbits, lblRabbitsKilled, lblCarrots, lblCarrotsStolen, lblCarrotsDestroyed, lblScore, lblSuurvivalTime

local lastKilled
local streakFrame
local streakKilled

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
        },
		caption = "",
    }
    lblRabbitsKilled = Chili.Label:New {
        right = 10,
        width = 100,
        y = 45,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    lblCarrots = Chili.Label:New {
        right = 10,
        width = 100,
        y = 125,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    lblCarrotsStolen = Chili.Label:New {
        right = 10,
        width = 100,
        y = 160,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    lblCarrotsDestroyed = Chili.Label:New {
        right = 10,
        width = 100,
        y = 195,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
	
    lblScore = Chili.Label:New {
        right = 10,
        width = 100,
        y = 250,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    lblSuurvivalTime = Chili.Label:New {
        right = 10,
        width = 100,
        y = 285,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    UpdateRabbits()
    UpdateCarrots()
end

function UpdateRabbits()
    local rabbitCount = Spring.GetTeamUnitDefCount(Spring.GetMyTeamID(), rabbitDefID)
    lblRabbits:SetCaption("\255\30\144\255Rabbits: " .. rabbitCount .. "\b")
	
	local rabbitsKilled = Spring.GetGameRulesParam("rabbits_killed") or 0
	if rabbitsKilled ~= 0 then
		lblRabbitsKilled:SetCaption("\255\30\144\255Killed: " .. rabbitsKilled .. "\b")
    else
        lblRabbitsKilled:SetCaption("")
    end
	local newKilled = lastKilled and (rabbitsKilled - lastKilled) or 0
	lastKilled = rabbitsKilled
	
	if newKilled > 0 then
		local frame = Spring.GetGameFrame()
		Spring.Echo(frame, streakFrame)
		if not streakFrame or (frame - streakFrame) > 120 then
			streakKilled = 0
		end
		streakFrame = frame
		local newStreakKilled = streakKilled + newKilled
		if newStreakKilled >= 20 and streakKilled < 20 then
			Spring.PlaySoundFile("sounds/godlike.ogg", 20)
			WG.AddEvent("GODLIKE!", 100, {1, 0 , 0, 1})
		elseif newStreakKilled >= 15 and streakKilled < 15 then
			Spring.PlaySoundFile("sounds/monsterkill.ogg", 20)
			WG.AddEvent("MonsterKill!!!", 80, {1, 0 , 0, 1})
		elseif newStreakKilled >= 8 and streakKilled < 8 then
			Spring.PlaySoundFile("sounds/ultrakill.ogg", 20)
			WG.AddEvent("UltraKill!", 60, {1, 0 , 0, 1})
		elseif newStreakKilled >= 3 and streakKilled < 3 then
			Spring.PlaySoundFile("sounds/killstreak.ogg", 20)
			WG.AddEvent("Killing Streak!", 40, {1, 0 , 0, 1})
		end
		streakKilled = newStreakKilled
	end
end

function UpdateCarrots()
    local carrotCount = Spring.GetGameRulesParam("carrot_count") or -1
    lblCarrots:SetCaption("\255\255\165\0Carrots: " .. carrotCount .. "\b")
	
	local carrotsStolen = Spring.GetGameRulesParam("carrots_stolen") or 0
	if carrotsStolen ~= 0 then
		lblCarrotsStolen:SetCaption("\255\255\165\0Stolen: " .. carrotsStolen .. "\b")
    else
        lblCarrotsStolen:SetCaption("")
    end
	
	local carrotsDestroyed = Spring.GetGameRulesParam("carrots_destroyed") or 0
	if carrotsDestroyed ~= 0 then
		lblCarrotsDestroyed:SetCaption("\255\255\165\0Destroyed: " .. carrotsDestroyed .. "\b")
    else
        lblCarrotsDestroyed:SetCaption("")
    end
end

function UpdateScores()
    local score = Spring.GetGameRulesParam("score") or 0
    lblScore:SetCaption("\255\255\255\0Score: " .. score .. "\b")
	
    local surivalTime = Spring.GetGameRulesParam("survivalTime") or 0
    lblSuurvivalTime:SetCaption("\255\255\255\0Time: " .. surivalTime .. "\b")
end

function widget:GameFrame()
    UpdateRabbits()
    UpdateCarrots()
	UpdateScores()
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