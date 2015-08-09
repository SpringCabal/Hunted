local unitName  =  "shotgun"

unitDef = {
  acceleration           = 5,
  airHoverFactor         = 0,
  brakeRate              = 5,
  buildCostEnergy        = 220,
  buildCostMetal         = 220,
  builder                = false,
  buildTime              = 220,
  canAttack              = true,
  canFly                 = true,
  canGuard               = true,
  canMove                = true,
  canPatrol              = true,
  canStop                = true,
  canSubmerge            = false,
  collide                = true,
  cruiseAlt              = 100,

  floater                = true,
  footprintX             = 2,
  footprintZ             = 2,
  hoverAttack            = true,
  idleAutoHeal           = 10,
  idleTime               = 150,
  maxDamage              = 860,
  maxVelocity            = 200,
  objectName             = "gun.s3o",
  script                 = unitName .. ".lua",
  turnRate               = 693,
}

return lowerkeys({ [unitName] = unitDef })
