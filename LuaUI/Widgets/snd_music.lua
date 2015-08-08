function widget:GetInfo()
  return {
    name      = "Music for dummies",
    desc      = "",
    author    = "ashdnazg",
    date      = "yesterday",
    license   = "GPL-v2",
    layer     = 1001,
    enabled   = true,
  }
end

local VOLUME = 2
local BUFFER = 0.015

local playingTime = 0
local dtTime = 0
local trackTime
local gameStarted = false
local musicFile = "LuaUI/sounds/Zeus_vs_Bunnies.ogg"

--function widget:GameStart()
function widget:Initialize()
    Spring.PlaySoundStream(musicFile, VOLUME)
    _, trackTime = Spring.GetSoundStreamTime()
    gameStarted = true
end


function widget:Update(dt)
    if gameStarted then
        playingTime = playingTime + dt
        --playingTime = Spring.GetSoundStreamTime()
        if playingTime > trackTime - BUFFER then
            Spring.PlaySoundStream(musicFile, VOLUME)
            playingTime = 0
        end
    end
end 

function widget:Shutdown()
    Spring.StopSoundStream()
end