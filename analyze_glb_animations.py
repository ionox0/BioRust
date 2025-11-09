#!/usr/bin/env python3
"""
GLB Animation Analyzer
Extracts animation information from GLB (GLTF Binary) files
"""

import json
import sys
from pathlib import Path
try:
    from pygltflib import GLTF2
except ImportError:
    print("Error: pygltflib not found. Install with: pip install pygltflib")
    sys.exit(1)

def analyze_glb_animations(file_path):
    """
    Analyze a GLB file and extract animation information
    """
    try:
        # Load the GLB file
        gltf = GLTF2.load(file_path)
        
        result = {
            'file_path': str(file_path),
            'file_name': Path(file_path).name,
            'animations': [],
            'total_animations': 0,
            'error': None
        }
        
        # Check if animations exist
        if not gltf.animations:
            result['total_animations'] = 0
            return result
        
        result['total_animations'] = len(gltf.animations)
        
        # Extract animation information
        for i, animation in enumerate(gltf.animations):
            anim_info = {
                'index': i,
                'name': animation.name if animation.name else f"Animation_{i}",
                'channels': len(animation.channels) if animation.channels else 0,
                'samplers': len(animation.samplers) if animation.samplers else 0,
                'duration': None,
                'target_nodes': [],
                'animation_types': set()
            }
            
            # Analyze channels and samplers to get more details
            if animation.channels and animation.samplers:
                max_time = 0
                
                for channel in animation.channels:
                    # Get target node info
                    if channel.target and channel.target.node is not None:
                        anim_info['target_nodes'].append(channel.target.node)
                    
                    # Get animation type (translation, rotation, scale)
                    if channel.target and channel.target.path:
                        anim_info['animation_types'].add(channel.target.path)
                    
                    # Get sampler for this channel
                    if channel.sampler is not None and channel.sampler < len(animation.samplers):
                        sampler = animation.samplers[channel.sampler]
                        
                        # Get input accessor (time values)
                        if sampler.input is not None and sampler.input < len(gltf.accessors):
                            input_accessor = gltf.accessors[sampler.input]
                            
                            # Calculate duration from time values
                            if input_accessor.max and len(input_accessor.max) > 0:
                                max_time = max(max_time, input_accessor.max[0])
                
                anim_info['duration'] = max_time if max_time > 0 else None
                anim_info['animation_types'] = list(anim_info['animation_types'])
                anim_info['target_nodes'] = list(set(anim_info['target_nodes']))
            
            result['animations'].append(anim_info)
        
        return result
        
    except Exception as e:
        return {
            'file_path': str(file_path),
            'file_name': Path(file_path).name,
            'animations': [],
            'total_animations': 0,
            'error': str(e)
        }

def guess_animation_purpose(name, duration=None, animation_types=None):
    """
    Try to guess the purpose of an animation based on its name and characteristics
    """
    name_lower = name.lower()
    
    # Common animation name patterns
    if any(keyword in name_lower for keyword in ['idle', 'rest', 'stand', 'default']):
        return "Idle/Rest animation"
    elif any(keyword in name_lower for keyword in ['walk', 'run', 'move', 'locomotion']):
        return "Movement animation"
    elif any(keyword in name_lower for keyword in ['attack', 'bite', 'strike', 'fight']):
        return "Attack animation"
    elif any(keyword in name_lower for keyword in ['death', 'die', 'dead']):
        return "Death animation"
    elif any(keyword in name_lower for keyword in ['jump', 'leap', 'hop']):
        return "Jump animation"
    elif any(keyword in name_lower for keyword in ['fly', 'flight', 'hover']):
        return "Flying animation"
    elif any(keyword in name_lower for keyword in ['eat', 'feed']):
        return "Eating animation"
    elif any(keyword in name_lower for keyword in ['turn', 'rotate']):
        return "Turning animation"
    elif any(keyword in name_lower for keyword in ['take_damage', 'hit', 'hurt']):
        return "Damage/Hit animation"
    elif any(keyword in name_lower for keyword in ['celebrate', 'victory', 'win']):
        return "Victory/Celebration animation"
    elif duration and duration < 1.0:
        return "Short action/transition animation"
    elif duration and duration > 5.0:
        return "Long loop/ambient animation"
    else:
        return "Unknown purpose"

def main():
    # List of GLB files to analyze
    glb_files = [
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/bee/bee-v1.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/beetle/black_ox_beetle_small.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/ladybug/ladybug.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/scorpion/scorpion.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/spider/spider_small.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/wolf_spider/wolf_spider.glb"
    ]
    
    results = []
    
    for file_path in glb_files:
        if not Path(file_path).exists():
            results.append({
                'file_path': file_path,
                'file_name': Path(file_path).name,
                'animations': [],
                'total_animations': 0,
                'error': f"File not found: {file_path}"
            })
            continue
        
        print(f"Analyzing: {Path(file_path).name}")
        result = analyze_glb_animations(file_path)
        results.append(result)
    
    # Print results
    print("\n" + "="*80)
    print("GLB ANIMATION ANALYSIS RESULTS")
    print("="*80)
    
    for result in results:
        print(f"\nFile: {result['file_name']}")
        print(f"Path: {result['file_path']}")
        
        if result['error']:
            print(f"❌ Error: {result['error']}")
            continue
        
        print(f"Total Animations: {result['total_animations']}")
        
        if result['total_animations'] == 0:
            print("  No animations found in this file.")
            continue
        
        for anim in result['animations']:
            print(f"\n  Animation #{anim['index'] + 1}:")
            print(f"    Name: {anim['name']}")
            if anim['duration']:
                print(f"    Duration: {anim['duration']:.3f} seconds")
            else:
                print(f"    Duration: Unknown")
            print(f"    Channels: {anim['channels']}")
            print(f"    Animation Types: {', '.join(anim['animation_types']) if anim['animation_types'] else 'None'}")
            print(f"    Purpose: {guess_animation_purpose(anim['name'], anim['duration'], anim['animation_types'])}")
        
        print("-" * 60)
    
    # Generate summary table
    print(f"\n{'='*80}")
    print("SUMMARY TABLE")
    print(f"{'='*80}")
    print(f"{'File':<25} {'Animations':<12} {'Animation Names':<40}")
    print("-" * 80)
    
    for result in results:
        file_name = result['file_name'][:24]  # Truncate long names
        anim_count = str(result['total_animations'])
        
        if result['error']:
            anim_names = "ERROR"
        elif result['total_animations'] == 0:
            anim_names = "None"
        else:
            anim_names = ", ".join([anim['name'] for anim in result['animations']])
            if len(anim_names) > 39:
                anim_names = anim_names[:36] + "..."
        
        print(f"{file_name:<25} {anim_count:<12} {anim_names:<40}")

if __name__ == "__main__":
    main()