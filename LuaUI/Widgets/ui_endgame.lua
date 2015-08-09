--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
  return {
    name      = "Chili EndGame Window",
    desc      = "Derived from v0.005 Chili EndGame Window by CarRepairer",
    author    = "Anarchid",
    date      = "April 2015",
    license   = "GNU GPL, v2 or later",
    layer     = 0,
    enabled   = true,
  }
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
local spSendCommands			= Spring.SendCommands

local echo = Spring.Echo

local caption

local Chili
local Image
local Button
local Checkbox
local Window
local Panel
local ScrollPanel
local StackPanel
local Label
local screen0
local color2incolor
local incolor2color

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local window_endgame
local frame_delay = 0

local function ShowEndGameWindow()
	screen0:AddChild(window_endgame)
end

local function SetupControls()
	window_endgame = Window:New{  
		name = "GameOver",
		caption = "Game Over",
		x = '40%',
		y = '40%',
		width  = '20%',
		height = '10%',
		padding = {8, 8, 8, 8};
		--autosize   = true;
		--parent = screen0,
		draggable = true,
		resizable = true,
		minWidth=500;
		minHeight=200;
	}
	
    local score = Spring.GetGameRulesParam("score") or 0 
    local survialTime = Spring.GetGameRulesParam("survivalTime") or 0 
--  	caption = Chili.Label:New{
--  		x = 20,
--  		y = 100,
--  		width = 100,
--  		parent = window_endgame,
--  		caption = "Overrun",
--  		fontsize = 40,
--  		textColor = {1,0,0,1},
--  	}
    
    caption = Chili.Label:New{
 		x = 20,
 		y = 40,
 		width = 100,
 		parent = window_endgame,
 		caption = "Score: " .. score .. "🐰",
 		fontsize = 40,
 		textColor = {1,0,0,1},
 	}
	caption = Chili.Label:New{
 		x = 20,
 		y = 85,
 		width = 100,
 		parent = window_endgame,
 		caption = "Time: " .. survialTime .. "🐰",
 		fontsize = 40,
 		textColor = {1,0,0,1},
 	}
	
	Button:New{
		y=40;
		width=80;
		right=50;
		height=40;
		caption="Exit",
		OnClick = {
			function() Spring.SendCommands("quit","quitforce") end
		};
		parent = window_endgame;
	}
    
   
    Button:New{
		y=80+20;
		width=80;
		right=50;
		height=40;
		caption="Restart",
		OnClick = {
			function() 
                Spring.SendCommands("cheat", "luarules reload", "cheat")
                window_endgame:Dispose()
                window_endgame = nil
                frame_delay = Spring.GetGameFrame()
            end
		};
		parent = window_endgame;
	}

    -- allows work with scened
--     local devMode = (tonumber(Spring.GetModOptions().play_mode) or 0) == 0
--     if devMode then
--         Button:New{
--             y=100;
--             width='80';
--             right=0;
--             height=40;
--             caption="Close me (dev mode)",
--             OnClick = {
--                 function() window_endgame:Dispose() end
--             };
--             parent = window_endgame;
--         }
--     end
    
	screen0:AddChild(window_endgame)

end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--callins
function widget:Initialize()
	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end
	

	Chili = WG.Chili
	Image = Chili.Image
	Button = Chili.Button
	Checkbox = Chili.Checkbox
	Window = Chili.Window
	Panel = Chili.Panel
	ScrollPanel = Chili.ScrollPanel
	StackPanel = Chili.StackPanel
	Label = Chili.Label
	screen0 = Chili.Screen0
	color2incolor = Chili.color2incolor
	incolor2color = Chili.incolor2color

end

function widget:GameFrame()
    local carrotCount = Spring.GetGameRulesParam("carrot_count") or -1
    if carrotCount == 0 then
        widget:GameOver({})
    end
end

function widget:GameOver(winningAllyTeams)
    if window_endgame or Spring.GetGameFrame() - frame_delay < 300 then
        return
    end
    local myAllyTeamID = Spring.GetMyAllyTeamID()
    for _, winningAllyTeamID in pairs(winningAllyTeams) do
        if myAllyTeamID == winningAllyTeamID then
            Spring.SendCommands("endgraph 0")
            SetupControls()
            caption:SetCaption("You win!");
            caption.font.color={0,1,0,1};
            ShowEndGameWindow()
            return
        end
    end
    Spring.SendCommands("endgraph 0")
    SetupControls()
    ShowEndGameWindow()
end


