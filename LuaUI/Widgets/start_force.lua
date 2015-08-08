

function widget:GetInfo()
	return {
		name    = 'Start Force',
		desc    = 'Forces the game to start',
		author  = 'GoogleFrog',
		date    = '8 August, 2015',
		license = 'GNU GPL v2',
        layer = 0,
		enabled = true,
	}
end

function widget:Initialize()
	Spring.SendCommands('forcestart')
	widgetHandler:RemoveWidget()
end