 include "lib_OS.lua"
 include "lib_UnitScript.lua"
 include "lib_Build.lua" 
--pieces
Head=piece"Head"
WiggleTail=piece"WiggleTail"
Body  =piece"Body"   
BLeg1 =piece"BLeg1"
BLeg2 =piece"BLeg2"
FLeg1 =piece"FLeg1"
FLeg2 =piece"FLeg2"


center=piece"center"
piecesTable=makeKeyPiecesTable(unitID, piece)

--Signals
local SIG_MOVE=2

function script.HitByWeapon (x, z, weaponDefID, damage) 
end


function script.Create()
	StartThread(MoveAnimationController)
end

function script.Killed(recentDamage, _)
	return 0
end



offSet=math.random(-0.2,0.2)
function MoveAnimation()
	Move(center, y_axis, 5, 15)

	Turn(Body  , x_axis, math.rad( -14),7+offSet) 
	Turn(BLeg1 ,x_axis, math.rad( 47+offSet),17+offSet) 
	Turn(BLeg2 ,x_axis, math.rad( 51),17+offSet) 
	Turn(FLeg1 ,x_axis, math.rad( 44),7+offSet) 
	
	Turn(FLeg2 ,x_axis, math.rad( 44),17+offSet) 

	Turn(Head  , x_axis, math.rad( -27),7+offSet)
													 
		Sleep(700)
	WaitForMove(center, y_axis)

	Move(center, y_axis, 0, 15)
	Turn(Body ,  x_axis,math.rad( -14),7+offSet) 
	Turn(BLeg1, x_axis,  math.rad( 9),7+offSet) 
	Turn(BLeg2, x_axis,  math.rad( 10),7+offSet) 
	Turn(FLeg1, x_axis,  math.rad( 10),7+offSet)  
	Turn(FLeg2, x_axis,  math.rad( 10),7+offSet) 
	Turn(Head, x_axis,  math.rad( 17 ),7+offSet)
		Sleep(300)
	WaitForMove(center, y_axis)
end

function MoveAnimationController()

	while true do
		if boolMoving == true then 
			MoveAnimation()
		end
			if boolMoving == false then
				reseT(piecesTable, 17)
			if maRa() == true then 
				idle()
			end
		end
		Sleep(100)
	end
end

function idle()
	val = math.random(1, 6)
	for i = 1, val do
		deg = math.random(10, 35)
		Turn(Head, y_axis, math.rad(deg), 9)
		Turn(WiggleTail, x_axis, math.rad(math.random(-10, 10), 8))
		WaitForTurn(Head, y_axis)
		Turn(Head, y_axis, math.rad(deg*-1), 9)

		Turn(WiggleTail, x_axis, math.rad(math.random(-10, 10), 8))
		WaitForTurn(Head, y_axis)
	end
	reseT(piecesTable)
end

function script.StartMoving()

	boolMoving = true
	Signal(SIG_MOVE)
end

function AttachCarrot(carrotID)
	Spring.UnitScript.AttachUnit(center, carrotID)
end

function DropCarrot(carrotID)
	Spring.UnitScript.DropUnit(carrotID)
end

function MoveEnded()
	Signal(SIG_MOVE)
	SetSignalMask(SIG_MOVE)
	Sleep(1500)
	boolMoving = false
end

function script.StopMoving()
	StartThread(MoveEnded)
end

function script.Killed()
		Explode(Body, SFX.EXPLODE_ON_HIT)
		Explode(BLeg1, SFX.EXPLODE_ON_HIT)
		Explode(BLeg2, SFX.EXPLODE_ON_HIT)
		Explode(FLeg1, SFX.EXPLODE_ON_HIT)
		Explode(FLeg2, SFX.EXPLODE_ON_HIT)
		return 1   -- spawn ARMSTUMP_DEAD corpse / This is the equivalent of corpsetype = 1; in bos
end
