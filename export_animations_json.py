#!/usr/bin/env python3
"""
Export animation data to JSON format
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
    """Analyze GLB animations and return detailed data"""
    try:
        gltf = GLTF2.load(file_path)
        
        result = {
            'file_path': str(file_path),
            'file_name': Path(file_path).name,
            'animations': [],
            'total_animations': 0,
            'error': None
        }
        
        if not gltf.animations:
            return result
        
        result['total_animations'] = len(gltf.animations)
        
        for i, animation in enumerate(gltf.animations):
            anim_info = {
                'index': i,
                'name': animation.name if animation.name else f"Animation_{i}",
                'channels': len(animation.channels) if animation.channels else 0,
                'samplers': len(animation.samplers) if animation.samplers else 0,
                'duration': None,
                'target_nodes': [],
                'animation_types': []
            }
            
            if animation.channels and animation.samplers:
                max_time = 0
                target_nodes = set()
                animation_types = set()
                
                for channel in animation.channels:
                    if channel.target and channel.target.node is not None:
                        target_nodes.add(channel.target.node)
                    
                    if channel.target and channel.target.path:
                        animation_types.add(channel.target.path)
                    
                    if channel.sampler is not None and channel.sampler < len(animation.samplers):
                        sampler = animation.samplers[channel.sampler]
                        
                        if sampler.input is not None and sampler.input < len(gltf.accessors):
                            input_accessor = gltf.accessors[sampler.input]
                            
                            if input_accessor.max and len(input_accessor.max) > 0:
                                max_time = max(max_time, input_accessor.max[0])
                
                anim_info['duration'] = max_time if max_time > 0 else None
                anim_info['animation_types'] = list(animation_types)
                anim_info['target_nodes'] = list(target_nodes)
            
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

def main():
    glb_files = [
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/bee/bee-v1.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/beetle/black_ox_beetle_small.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/ladybug/ladybug.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/scorpion/scorpion.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/spider/spider_small.glb",
        "/Users/ianjohnson/Desktop/code/Bug_Game/client/resources/bugs/wolf_spider/wolf_spider.glb"
    ]
    
    results = {}
    
    for file_path in glb_files:
        if Path(file_path).exists():
            print(f"Processing: {Path(file_path).name}")
            result = analyze_glb_animations(file_path)
            results[Path(file_path).stem] = result
    
    # Export to JSON
    output_file = "/Users/ianjohnson/Desktop/code/rust-game/glb_animations.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    
    print(f"\nDetailed animation data exported to: {output_file}")

if __name__ == "__main__":
    main()