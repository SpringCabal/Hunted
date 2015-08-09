local base = piece "base"

function script.Create()
	Turn(base, x_axis, math.pi/2)
	Move(base, y_axis, -7)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end