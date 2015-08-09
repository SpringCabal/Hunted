return {
  ["fuse"] = {
    fuse = {
      air                = true,
      class              = [[CSimpleParticleSystem]],
      count              = 1,
      ground             = true,
      properties = {
        airdrag            = 0.87,
        alwaysvisible      = true,
        colormap           = [[0.9 0.9 0.9 0.9	0.6 0.6 0.6 0.0]],
        directional        = false,
        emitrot            = 90,
        emitrotspread      = 5,
        emitvector         = [[0, 1, 0]],
        gravity            = [[0, 0.1, 0]],
        numparticles       = 3,
        particlelife       = 2,
        particlelifespread = 4,
        particlesize       = 2,
        particlesizespread = 1,
        particlespeed      = 0.6,
        particlespeedspread = 0.4,
        pos                = [[0, 1, 0]],
        sizegrowth         = 0.5,
        sizemod            = 1.0,
        texture            = [[spattercloud]],
        useairlos          = true,
      },
    },
  },

}

