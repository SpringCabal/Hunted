-- Explicitly empty explosion generator, used by the NOWEAPON fallback weapon so
-- that units without an explodeAs weapon do not log a missing CEG warning.
return {
	["nothing"] = {},
}
