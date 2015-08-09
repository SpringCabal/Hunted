local base = piece "base"

local SIG_STEAL = 1
local riseSpeed = 6.5
local x,y,z

local function StolenThread(progress)
	Signal(SIG_STEAL)
	SetSignalMask(SIG_STEAL)
	while true do
		Spring.PlaySoundFile("sounds/digitout.wav", 0.25, x, y, z)
		Spring.SpawnCEG("dirtfling", x, y, z)
		Sleep(500)
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
	x,y,z = Spring.GetUnitPosition(unitID)
	Move(base, y_axis, -10)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end