local base = piece "base"

local SIG_STEAL = 1
local riseSpeed = 6.5
local x,y,z

local function Sound()
	SetSignalMask(SIG_STEAL)
	while true do
		if math.random() < 0.7 then
			Spring.PlaySoundFile("sounds/digitout.wav", 4, x, y, z)
		end
		Sleep(600)
	end
end

local function StolenThread(progress)
	Signal(SIG_STEAL)
	SetSignalMask(SIG_STEAL)
	StartThread(Sound)
	while true do
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
	x,_,z = Spring.GetUnitPosition(unitID)
	y = Spring.GetGroundHeight(x,z) + 1
	Move(base, y_axis, -10)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end