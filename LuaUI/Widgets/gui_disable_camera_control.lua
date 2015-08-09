--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
	return {
		name      = "Disable camera control",
		desc      = "Disables camera zooming and panning",
		author    = "gajop",
		date      = "WIP",
		license   = "GPLv2",
		version   = "0.1",
		layer     = -1000,
		enabled   = true,  --  loaded by default?
		handler   = true,
		api       = true,
		hidden    = true,
	}
end

function widget:Initialize()
    for k, v in pairs(Spring.GetCameraState()) do
        print(k .. " = " .. tostring(v) .. ",")
    end
--     local devMode = (tonumber(Spring.GetModOptions().play_mode) or 0) == 0
--     if devMode then
--         widgetHandler:RemoveWidget(widget)
--         return
--     end
    s = {
        dist = 1389.7800292969,
        px = 1902.5513916016,
        py = 103.51550292969,
        pz = 2218.6779785156,
        rz = 0,
        dx = 0,
        dy = -0.8170080780983,
        dz = -0.57662618160248,
        fov = 45,
        ry = 0,
        mode = 2,
        rx = 2.5269994735718,
        name = "spring",
    }
    Spring.SetCameraState(s, 0)
end

function widget:Shutdown()
end

function widget:MouseWheel(up,value)
    -- uncomment this to disable zoom/panning
    return true
end