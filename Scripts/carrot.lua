local base = piece "base"

local SIG_STEAL = 0
local riseSpeed = 6.5
local x,y,z = Spring.GetUnitPosition(unitID)

local function StolenThread(progress)
	Signal(SIG_STEAL)
	SetSignalMask(SIG_STEAL)
	while true do
		Spring.PlaySoundFile("sounds/digitout.wav", 1, x, y, z)
		Spring.SpawnCEG("dirtfling", x, y, z)
		Sleep(128)
	end
end

function StartBeingStolen(progress)
	StartThread(StolenThread)
	Move(base, y_axis, -10 + riseSpeed*progress/30)
	Move(base, y_axis, 20, riseSpeed)
end

function StopBeingStolen(progress)
	Signal(SIG_STEAL)
	Move(base, y_axis, -10 + riseSpeed*progress/30)
	Move(base, y_axis, -10 + riseSpeed*progress/30, 0.0001)
end


function script.Create()
	Move(base, y_axis, -10)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end