-- $Id$
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local unitInfo = {}

local distanceID
local timeID

local function DrawUnit(unitid, material, materialID)
    if distanceID == nil then
        distanceID = gl.GetUniformLocation(material.shader, "distance")
    end
    if timeID == nil then
        timeID = gl.GetUniformLocation(material.shader, "time")
    end

    local fx, fz, fsize = GG.flashlightPos.x, GG.flashlightPos.z, GG.flashlightPos.size
    local x, _, z = Spring.GetUnitPosition(unitid)

    local dx = math.abs(x - fx)
    local dz = math.abs(z - fz)
    local d = math.sqrt(dx * dx + dz * dz)

    local d1 = math.max(d - fsize / 2, 0)
    local d2 = math.max(d - fsize, 0)

    local distance = d1
    if d2 > 10 then
        distance = 100
    end

    gl.Uniform(distanceID, distance / 100)
    gl.Uniform(timeID, Spring.GetGameFrame()%360)
--   local info = unitInfo[unitid]
--   if (not info) then
--     info = {dir=0, lx=0, lz=0}
--     unitInfo[unitid] = info
--   end
-- 
--   local vx,vy,vz = Spring.GetUnitVelocity(unitid)
--   local speed = (vx*vx+vy*vy+vz*vz)^0.5
-- 
--   local curFrame = Spring.GetGameFrame()
--   if (info.n ~= curFrame) then
--     info.n = curFrame;
--     local lx = info.lx
--     local lz = info.lz
--     local dx,dy,dz = Spring.GetUnitDirection(unitid)
--     info.dir =  info.dir*0.95 + (lx*dz - lz*dx) / ( (lx*lx+lz*lz)^0.5 + (dx*dx+dz*dz)^0.5 );
--     info.lx,info.lz = dx,dz;
--   end
-- 
--   gl.Uniform(material.frameLoc, Spring.GetGameFrame()%360)
--   gl.Uniform(material.speedLoc, info.dir,0,speed)

  return false --// engine should still draw it (we just set the uniforms for the shader)
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local materials = {
   rabbit = {
      shader    = include("ModelMaterials/Shaders/rabbit.lua"),
      force     = true, --// always use the shader even when normalmapping is disabled
      usecamera = false,
      culling   = GL.BACK,
      texunits  = {
        [0] = '%%UNITDEFID:0',
        [1] = '%%UNITDEFID:1',
        [2] = '$shadow',
        [3] = '$specular',
        [4] = '$reflection',
      },
      DrawUnit = DrawUnit
   }
}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- affected unitdefs

local unitMaterials = {
   rabbit = "rabbit",
}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

return materials, unitMaterials

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
