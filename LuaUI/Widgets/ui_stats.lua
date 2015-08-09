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

local lblRabbits, lblRabbitsKilled
local lblCarrots, lblCarrotsStolen, lblCarrotsDestroyed
local lblScore, lblSuurvivalTime
local lblShotgun, lblMine

local lastKilled
local streakFrame
local streakKilled

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Updating

local function UpdateRabbits()
    local rabbitCount = Spring.GetTeamUnitDefCount(Spring.GetMyTeamID(), rabbitDefID)
    lblRabbits:SetCaption("\255\30\144\255Rabbits: " .. rabbitCount .. "\b")
	
	local rabbitsKilled = Spring.GetGameRulesParam("rabbits_killed") or 0
	lblRabbitsKilled:SetCaption("\255\30\144\255Kills: " .. rabbitsKilled .. "\b")
	
	local newKilled = lastKilled and (rabbitsKilled - lastKilled) or 0
	lastKilled = rabbitsKilled
	
	if newKilled > 0 then
		local frame = Spring.GetGameFrame()
		if not streakFrame or (frame - streakFrame) > 120 then
			streakKilled = 0
		end
		streakFrame = frame
		local newStreakKilled = streakKilled + newKilled
		if newStreakKilled >= 35 and streakKilled < 35 then
			Spring.PlaySoundFile("sounds/godlike.ogg", 20)
			WG.AddEvent("GODLIKE!", 100, {1, 0 , 0, 1})
		elseif newStreakKilled >= 20 and streakKilled < 20 then
			Spring.PlaySoundFile("sounds/monsterkill.ogg", 20)
			WG.AddEvent("MonsterKill!!!", 80, {1, 0 , 0, 1})
		elseif newStreakKilled >= 12 and streakKilled < 12 then
			Spring.PlaySoundFile("sounds/ultrakill.ogg", 20)
			WG.AddEvent("UltraKill!", 60, {1, 0 , 0, 1})
		elseif newStreakKilled >= 5 and streakKilled < 5 then
			Spring.PlaySoundFile("sounds/killstreak.ogg", 20)
			WG.AddEvent("Killing Streak!", 40, {1, 0 , 0, 1})
		end
		streakKilled = newStreakKilled
	end
end

local function UpdateCarrots()
    local carrotCount = Spring.GetGameRulesParam("carrot_count") or -1
    lblCarrots:SetCaption("\255\255\165\0Carrots: " .. carrotCount .. "\b")
end

local function UpdateScores()
    local score = Spring.GetGameRulesParam("score") or 0
    lblScore:SetCaption("\255\255\255\0Score: " .. score .. "\b")
	
    local survivalTime = Spring.GetGameRulesParam("survivalTime") or 0
    lblSuurvivalTime:SetCaption("\255\255\255\0Time: " .. survivalTime .. "\b")
end

local function UpdateAmmo()
    local shotgunAmmo = math.floor(Spring.GetGameRulesParam("shotgun_ammo") or 0)
    lblShotgun:SetCaption("\255\255\255\0Ammo: " .. shotgunAmmo .. "\b")
	
    local mineAmmo = math.floor(Spring.GetGameRulesParam("mine_ammo") or 0)
    lblMine:SetCaption("\255\255\255\0Mines: " .. mineAmmo .. "\b")
end

function UpdateFlash()
    local value = math.floor(Spring.GetGameRulesParam("flare_progress") or 0)
    pbFlash:SetValue(value)
end

function widget:GameFrame()
    UpdateRabbits()
    UpdateCarrots()
	UpdateScores()
	UpdateAmmo()
    UpdateFlash()
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Display Managment

function widget:Initialize()
	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end

	Chili = WG.Chili
	screen0 = Chili.Screen0
	
	local screenWidth, screenHeight = Spring.GetWindowGeometry()
	
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
        x = screenWidth/2 - 60,
        width = 100,
        y = 10,
        height = 50,
		align = "left",
        parent = screen0,
        font = {
            size = 32,
        },
		caption = "",
    }
	
    lblScore = Chili.Label:New {
        right = 10,
        width = 100,
        bottom = 45,
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
        bottom = 10,
        height = 50,
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }

    lblFlash = Chili.Label:New {
        right = "45%",
        width = "10%",
        bottom = 30,
        height = 50,
        parent = screen0,
        font = {
            size = 20,
        },
        align = "center",
		caption = "\255\147\112\219Flash\b",
    }
    pbFlash = Chili.Progressbar:New {
        value = 0,
        right = "45%",
        width = "10%",
        bottom = 20,
        height = 30,
        parent = screen0,
    }
    lblShotgun = Chili.Label:New {
        x = 10,
        width = 100,
        y = 10,
        height = 50,
		align = "left",
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }
    lblMine = Chili.Label:New {
        x = 10,
        width = 100,
        y = 45,
        height = 50,
		align = "left",
        parent = screen0,
        font = {
            size = 24,
        },
		caption = "",
    }

	
    UpdateRabbits()
    UpdateCarrots()
	UpdateScores()
	UpdateAmmo()
    UpdateFlash()
end