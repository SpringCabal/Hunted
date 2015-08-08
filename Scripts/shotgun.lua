local shell = piece "Shell"
local shotgun = piece "Shotgun"

function script.Create()
	Hide(shell)
	Turn(shotgun, x_axis, 0)
	Turn(shotgun, y_axis, 0)
	Turn(shotgun, z_axis, 0)
end

function SetPitch(pitch)
	Turn(shotgun, x_axis, pitch)
end


function Fire()
	Explode(shell, 0)
end
