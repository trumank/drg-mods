// Copyright Epic Games, Inc. All Rights Reserved.

#include "BPGenCommands.h"

#define LOCTEXT_NAMESPACE "FBPGenModule"

void FBPGenCommands::RegisterCommands()
{
	UI_COMMAND(OpenPluginWindow, "BPGen", "Bring up BPGen window", EUserInterfaceActionType::Button, FInputGesture());
}

#undef LOCTEXT_NAMESPACE
