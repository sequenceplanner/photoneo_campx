from flask import Flask, request, jsonify
import subprocess
import os

app = Flask(__name__)

# Base directory where executables are located
BASE_DIR = '/opt/app/build_all'

# Mapping of endpoint to executable
EXECUTABLES = {
    'connect': os.path.join(BASE_DIR, 'Connect', 'connect_app'),    # Replace 'connect_app' with your actual executable names
    'capture': os.path.join(BASE_DIR, 'Capture', 'capture_app'),
    'freerun': os.path.join(BASE_DIR, 'Freerun', 'freerun_app'),
    'stop': os.path.join(BASE_DIR, 'Stop', 'stop_app')
}

@app.route('/execute/<command>', methods=['POST'])
def execute_command(command):
    if command not in EXECUTABLES:
        return jsonify({'error': 'Invalid command'}), 400

    # Get arguments from the JSON body
    data = request.get_json()
    args = data.get('args', []) if data else []

    if not isinstance(args, list):
        return jsonify({'error': 'Args must be a list'}), 400

    # Construct the command
    cmd = [EXECUTABLES[command]] + args

    try:
        # Execute the command and capture stdout and stderr
        result = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60  # Set a timeout for command execution
        )

        response = {
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }

        return jsonify(response), 200

    except subprocess.TimeoutExpired:
        return jsonify({'error': 'Command timed out'}), 504
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/')
def index():
    return jsonify({'message': 'C++ Executable API Server'}), 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
