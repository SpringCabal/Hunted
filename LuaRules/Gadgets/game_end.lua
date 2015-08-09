--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name      = "Game End",
		desc      = "Handles team/allyteam deaths and declares gameover",
		author    = "Andrea Piras",
		date      = "June, 2013",
		license   = "GNU GPL, v2 or later",
		layer     = 0,
		enabled   = true  --  loaded by default?
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--SYNCED
if (not gadgetHandler:IsSyncedCode()) then
   return false
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local initializeFrame = 0

function gadget:Initialize()
	initializeFrame = Spring.GetGameFrame() or 0
end

function gadget:GameFrame(frame)
	if frame > initializeFrame + 2 then
		local carrotCount = Spring.GetGameRulesParam("carrot_count")
		
        -- We're doing widget-only game overs which makes restarts easier
-- 		if carrotCount <= 0 then
-- 			Spring.GameOver({})
-- 		end
	end
end