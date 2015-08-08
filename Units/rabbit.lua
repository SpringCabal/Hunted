 

local unitName  =  "rabbit"

local unitDef  =  {
--Internal settings
    ObjectName = "rabbitProto.s3o",
    name = "Rabbit",
    UnitName = unitName,
    script = "rabbit.lua",
    
--Unit limitations and properties
    MaxDamage = 800,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
    
--Pathfinding and related
    Acceleration = 0.7,
    BrakeRate = 1.0,
    FootprintX = 2,
    FootprintZ = 2,
    MaxSlope = 15,
    MaxVelocity = 5.0,
    MaxWaterDepth = 20,
    MovementClass = "Bot2x2",
    TurnRate = 900,
    
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
--    collisionVolumeOffsets    =  "0 0 0",
--    collisionVolumeScales     =  "20 20 20",
--    collisionVolumeTest       =  1,
--    collisionVolumeType       =  "box",
}

return lowerkeys({ [unitName]  =  unitDef })