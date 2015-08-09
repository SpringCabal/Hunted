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

LOG_SECTION = "flashlight"

-- unsynced
if not Script.GetSynced() then

VFS.Include("LuaUI/widgets/glVolumes.lua")

flashlights = {}

local GUN_FLASHLIGHT_SIZE = 150
local GUN_FLASHLIGHT_COLOR = {0.69, 0.61, 0.85, 0.3}
-- purpleish
-- 0.69, 0.61, 0.85, 0.3
-- yellowish
-- 0.941, 0.901, 0.549, 0.3

local gunFlashlightID

function gadget:Update()
    local x, y = Spring.GetMouseState()
    local result, coords = Spring.TraceScreenRay(x, y, true)
    if result == "ground" then
        if gunFlashlightID == nil then
            gunFlashlightID = CreateFlashlight(coords[1], coords[3])
        else
            UpdateFlashlight(gunFlashlightID, coords[1], coords[3])
        end
    end
end

function gadget:DrawWorld()
    gl.PushMatrix()
    for _, f in pairs(flashlights) do
        local x, z, size, c = f.x, f.z, f.size, f.color
        gl.Color(c[1], c[2], c[3], c[4])
        if x ~= nil then
            gl.Utilities.DrawGroundCircle(x, z, size)
        end
    end
    gl.PopMatrix()
end

-- (optional) size
-- (optional) color is an array in format {r, g, b, a}
function CreateFlashlight(x, z, size, color)
    local id = #flashlights + 1
    flashlights[id] = {}
    UpdateFlashlight(id, x, z, size, color)
    return id
end

-- (optional) size
-- (optional) color is an array in format {r, g, b, a}
function UpdateFlashlight(id, x, z, size, color)
    if not flashlights[id] then
        Spring.Log(LOG_SECTION, LOG.ERROR, "No flashlight with id: " .. tostring(id))
        return
    end
    flashlights[id] = {
        x = x,
        z = z,
        size = size or GUN_FLASHLIGHT_SIZE,
        color = color or GUN_FLASHLIGHT_COLOR,
    }
end

function RemoveFlashlight(id)
    if not flashlights[id] then
        Spring.Log(LOG_SECTION, LOG.ERROR, "No flashlight with id: " .. tostring(id))
        return
    end
    flashlights[id] = nil
end

function _CreateFlashlight(_, ...) CreateFlashlight(...) end
function _UpdateFlashlight(_, ...) UpdateFlashlight(...) end
function _RemoveFlashlight(_, ...) RemoveFlashlight(...) end

function gadget:Initialize()
    gadgetHandler:AddSyncAction("CreateFlashlight", _CreateFlashlight)
    gadgetHandler:AddSyncAction("UpdateFlashlight", _UpdateFlashlight)
    gadgetHandler:AddSyncAction("RemoveFlashlight", _RemoveFlashlight)
end

-- direct access is also available in UNSYNCED (useful for shaders)
GG.flashlights = flashlights

else
    -- SYNCED API
    GG.Flashlight = {
        Create = function(...) SendToUnsynced("CreateFlashlight", ...) end,
        Update = function(...) SendToUnsynced("UpdateFlashlight", ...) end,
        Remove = function(...) SendToUnsynced("RemoveFlashlight", ...) end,
    }
end
