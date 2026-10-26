-- Exact in float32 Lua_Number. Keep presentation RNG independent per unit.
return function(unit)
    local state = unit % 65521
    return function(lower, upper)
        state = (state * 251 + 1) % 65521
        local value = state / 65521
        if lower ~= nil then return lower + math.floor(value * (upper - lower + 1)) end
        return value
    end
end
