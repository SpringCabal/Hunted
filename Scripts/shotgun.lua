local shell = piece "Shell"
local shotgun = piece "Shotgun"
local ReloadAnim = piece "ReloadAnim"
local KickBackRotator = piece "KickBackRotator"
local SIG_RELOAD = 2

function script.Create()
	Hide(shell)
	Turn(shotgun, x_axis, 0)
	Turn(shotgun, y_axis, 0)
	Turn(shotgun, z_axis, 0)
end

local LastPitch=0
function SetPitch(pitch)
	Signal(SIG_RELOAD)
	LastPitch=pitch
	Turn(shotgun, x_axis, pitch)
end

function reloadAnimation()
	Explode(shell, 0)
	Signal(SIG_RELOAD)
	SetSignalMask(SIG_RELOAD)
	Turn(KickBackRotator,x_axis,math.rad(-29),80)
	Move(ReloadAnim,z_axis,-10,45)
	WaitForMove(ReloadAnim,z_axis)
	Turn(KickBackRotator,x_axis,math.rad(LastPitch),55)
	Move(ReloadAnim,z_axis,0,55)
end

function Fire()
StartThread(reloadAnimation)
end
