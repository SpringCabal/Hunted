local unitName  =  "carrot_carried"

local unitDef  =  {
--Internal settings
    ObjectName = "carrot.s3o",
    name = "Carrot",
    UnitName = unitName,
    script = unitName .. ".lua",
    
--Unit limitations and properties
    MaxDamage = 800,
    RadarDistance = 0,
    SightDistance = 400,
    Upright = 0,
    
--Pathfinding and related
    Acceleration = 0.2,
    BrakeRate = 0.1,
    FootprintX = 1,
    FootprintZ = 1,
    MaxSlope = 15,
    MaxVelocity = 1,
    MaxWaterDepth = 20,
    MovementClass = "Bot2x2",
    TurnRate = 500,
    
--Abilities
    Builder = 0,
    CanAttack = 1,
    CanGuard = 0,
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