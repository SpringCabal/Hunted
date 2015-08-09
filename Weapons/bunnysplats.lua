--------------------------------------------------------------------------------
-- These represent both the area of effect and the camerashake amount
local smallExplosion = 0
local smallExplosionImpulseFactor = 0
local mediumExplosion = 0
local mediumExplosionImpulseFactor = 0
local largeExplosion = 0
local largeExplosionImpulseFactor = 0
local hugeExplosion = 0
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
