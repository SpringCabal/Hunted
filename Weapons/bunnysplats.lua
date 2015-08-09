--------------------------------------------------------------------------------
-- These represent both the area of effect and the camerashake amount
local smallExplosion = 200
local smallExplosionImpulseFactor = 0
local mediumExplosion = 400
local mediumExplosionImpulseFactor = 0
local largeExplosion = 600
local largeExplosionImpulseFactor = 0
local hugeExplosion = 1000
local hugeExplosionImpulseFactor = 0

unitDeaths = {
	smallBunnyExplosion = {
		weaponType		   = "Cannon",
		impulseFactor      = smallExplosionImpulseFactor,
		soundstart = "splatsplatter1.wav",
		AreaOfEffect=smallExplosion,
		explosiongenerator="custom:genericbunnyexplosion-small-red",
		cameraShake=smallExplosion,
		damage = {
			default            = 0,
		},
	},
}

return lowerkeys(unitDeaths)
