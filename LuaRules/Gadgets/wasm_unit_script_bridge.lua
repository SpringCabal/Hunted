function gadget:GetInfo()
    return {
        name = "WasmUnitScriptBridge",
        desc = "Named-call adapters for the unchanged Lua unit scripts",
        author = "SpringCabal",
        license = "GNU GPL, v2 or later",
        layer = 1, -- after the layer-0 Lua unit-script framework
        enabled = true,
    }
end

if not gadgetHandler:IsSyncedCode() then return false end

local exports = {
    [UnitDefNames.rabbit.id] = {"AttachCarrot", "DropCarrot"},
    [UnitDefNames.carrot.id] = {"StartBeingStolen", "StopBeingStolen"},
    [UnitDefNames.carrot_carried.id] = {"BeDropped"},
    [UnitDefNames.mine.id] = {"StartFuse"},
    [UnitDefNames.shotgun.id] = {"Fire", "SetPitch"},
}

local function register(unitID, name, fn)
    Spring.UnitScript.UpdateCallIn(unitID, name, function(...)
        -- The Lua framework maintains an active-unit stack in addition to
        -- the engine's context. Preserve it for StartThread/Signal/Sleep.
        Spring.UnitScript.CallAsUnit(unitID, fn, ...)
        return 1
    end)
end

local function destroyLater(unitID, param)
    Spring.UnitScript.StartThread(function()
        Spring.UnitScript.Sleep(1)
        Spring.SetUnitRulesParam(unitID, param, 1)
        Spring.DestroyUnit(unitID, false, false)
    end)
end

function gadget:UnitCreated(unitID, unitDefID)
    local env = Spring.UnitScript.GetScriptEnv(unitID)
    if not env then return end
    for _, name in ipairs(exports[unitDefID] or {}) do
        local fn = env[name]
        if fn then register(unitID, name, fn) end
    end
    if unitDefID == UnitDefNames.carrot.id or unitDefID == UnitDefNames.carrot_dropped.id then
        register(unitID, "ConsumeCarrot", function() destroyLater(unitID, "internalDestroy") end)
    elseif unitDefID == UnitDefNames.carrot_carried.id then
        register(unitID, "ScoreCarrot", function() destroyLater(unitID, "carrotScored") end)
    end
    if unitDefID == UnitDefNames.carrot_dropped.id then
        -- Lua's AI calls these optional visual hooks only when present.
        register(unitID, "StartBeingStolen", function() end)
        register(unitID, "StopBeingStolen", function() end)
    end
end

function gadget:Initialize()
    -- Lua's lighthouse owns this mutable beam-area record. WASM reads its
    -- published lighthouse_x/z parameters instead of Lua's old AI registry.
    GG.AddScaryArea = function(area) return area end
    GG.RemoveScaryArea = function(area) area.inactive = true end
    for _, unitID in ipairs(Spring.GetAllUnits()) do
        self:UnitCreated(unitID, Spring.GetUnitDefID(unitID))
    end
end
