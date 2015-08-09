local Sounds = {
  SoundItems = {
    IncomingChat = {
      --- always play on the front speaker(s)
      file = "sounds/beep4.wav",
      in3d = "false",
    },
    MultiSelect = {
      --- always play on the front speaker(s)
      file = "sounds/button9.wav",
      in3d = "false",
    },
    MapPoint = {
      --- respect where the point was set, but don't attenuate in distance
      --- also, when moving the camera, don't pitch it
      file = "sounds/beep6.wav",
      rolloff = 0,
      dopplerscale = 0,
    },
    digitout = {
      --- some things you can do with this file
      --- can be either ogg or wav
      file = "sounds/digitout.wav",
	  in3d = "true",
    },
  },
}

return Sounds