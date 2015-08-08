

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
	Spring.SendCommands('cheat')
	Spring.SendCommands('globallos')
	Spring.SendCommands('cheat')
end