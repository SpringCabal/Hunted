

function widget:GetInfo()
	return {
		name    = 'Start Force',
		desc    = 'Forces the game to start and sets globallos on',
		author  = 'GoogleFrog',
		date    = '8 August, 2015',
		license = 'GNU GPL v2',
        layer = 0,
		enabled = true,
	}
end

function widget:Initialize()
	Spring.SendCommands('forcestart')
end

function widget:GameStart()
	-- Kill healthbars.
	Spring.SendCommands({"showhealthbars 0", "showrezbars 0", "unbind f9 showhealthbars"})
	
	local cheat = Spring.IsCheatingEnabled()
	if not cheat then
		Spring.SendCommands('cheat')
	end
	Spring.SendCommands('globallos')
	if not cheat then
		Spring.SendCommands('cheat')
	end
	Spring.SendCommands('disticon 99999')
end