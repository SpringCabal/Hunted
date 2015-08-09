local base = piece "base"

local dropDefID = UnitDefNames["carrot_dropped"].id

function script.Create()
	Turn(base, x_axis, math.pi*0.4)
	Move(base, y_axis, 7.5)
end

local function BeDroppedThread()
	Sleep(500)
	Move(base, y_axis, -100, nil, true)
	local x,y,z = Spring.GetUnitPosition(unitID)
	local dx, dy, dz = Spring.GetUnitDirection(unitID)
	
	local droppedID = Spring.CreateUnit(dropDefID, x, y, z, 0, 0, false, false)
	Spring.SetUnitDirection(droppedID, dx, dy, dz)
	
	Spring.DestroyUnit(unitID, false, false)
end

function BeDropped()
	Move(base, y_axis, -7, 29)
	Turn(base, x_axis, math.pi*0.5, math.pi*0.2)
	StartThread(BeDroppedThread)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end