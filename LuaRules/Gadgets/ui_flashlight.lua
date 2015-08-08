function gadget:GetInfo()
  return {
    name      = "Flashlight effect",
    desc      = "Flashlight effect for Hunted",
    author    = "gajop",
    date      = "August 2015",
    license   = "GNU GPL, v2 or later",
    layer     = 0,
    enabled   = true,
  }
end

-- unsynced
if not Script.GetSynced() then

VFS.Include("LuaUI/widgets/glVolumes.lua")

local FLASHLIGHT_SIZE = 100

function gadget:Update()
    local x, y = Spring.GetMouseState()
    local result, coords = Spring.TraceScreenRay(x, y, true)
    if result == "ground" then
        self.x = coords[1]
        self.z = coords[3]
        GG.flashlightPos = {
            x = self.x,
            z = self.z,
            size = FLASHLIGHT_SIZE,
        }
    end
end

function gadget:DrawWorld()
    gl.PushMatrix()
    gl.Color(0.69, 0.61, 0.85, 0.3) 
    if self.x ~= nil then
        gl.Utilities.DrawGroundCircle(self.x, self.z, FLASHLIGHT_SIZE)
    end
    gl.PopMatrix()
end


end