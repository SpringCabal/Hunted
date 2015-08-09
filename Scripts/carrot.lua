local base = piece "base"

local SIG_STEAL = 0
local riseSpeed = 6.5

local function StolenThread(progress)
	Signal(SIG_STEAL)
	SetSignalMask(SIG_STEAL)
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