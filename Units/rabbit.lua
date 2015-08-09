local unitName  =  "rabbit"

local unitDef  =  {
--Internal settings
    ObjectName = "rabbit.s3o",
    name = "Rabbit",
    UnitName = unitName,
    script = unitName .. ".lua",
	explodeAs = "smallBunnyExplosion",
    
--Unit limitations and properties
    MaxDamage = 800,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
	
-- Transporting
	releaseHeld = true,
	holdSteady = true,
	transportCapacity = 10,
	transportSize = 10,
	
--Pathfinding and related
    Acceleration = 0.2,
    BrakeRate = 0.1,
    FootprintX = 2,
    FootprintZ = 2,
    MaxSlope = 15,
    MaxVelocity = 32, -- max velocity is so high because Spring cannot increase max velocity beyond its maximum.
    MaxWaterDepth = 20,
    MovementClass = "Bot2x2",
	TurnInPlace = false,
	TurnInPlaceSpeedLimit = 1.6, 
	turnInPlaceAngleLimit = 90,
    TurnRate = 3000,
	
	customParams = {
		turnaccel = 500
	},
    
--Abilities
    Builder = 0,
    CanAttack = 1,
    CanGuard = 1,
    CanMove = 1,
    CanPatrol = 1,
    CanStop = 1,
    LeaveTracks = 0,
    Reclaimable = 0,
	
--Hitbox
	collisionVolumeOffsets    =  "0 16 0",
	collisionVolumeScales     =  "64 64 64",
	collisionVolumeType       =  "sphere",
}

return lowerkeys({ [unitName]  =  unitDef })