local function FuseThread()
	local x,_,z = Spring.GetUnitPosition(unitID)
	local y = Spring.GetGroundHeight(x,z) + 50
	local t = 90
	while true do
		local hor = 11*(t/90)^2
		Spring.SpawnCEG("fuse", x + hor, y + t*0.33, z - hor)
		t = t - 1
		Sleep(33)
	end
end

function StartFuse()
	StartThread(FuseThread)
end

function script.Killed(recentDamage, maxHealth)
	return 0
end