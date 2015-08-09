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

    local minDistance = math.huge
    local flare_state = Spring.GetGameRulesParam("flare_state")
    if flare_state == "active" then 
        minDistance = 0
    else
        for _, flashlight in pairs(GG.flashlights) do
            local fx, fz, fsize = flashlight.x, flashlight.z, flashlight.size
            local x, _, z = Spring.GetUnitPosition(unitid)

            local dx = math.abs(x - fx)
            local dz = math.abs(z - fz)
            local d = math.sqrt(dx * dx + dz * dz)

            local d1 = math.max(d - 1.1*fsize / 2, 0)
--             local d2 = math.max(d - fsize, 0)

            local distance = d1

            if minDistance > distance then
                minDistance = distance
            end
        end
    end

--     if minDistance > 80 then
--         return true
--     end
    gl.Uniform(distanceID, minDistance / 100)
    gl.Uniform(timeID, Spring.GetGameFrame()%360)

    if minDistance > 90 then
        return true
    end
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
