#!/usr/bin/env python3
import pika
import json
import uuid
import sys

def test_service():
    # Connect to RabbitMQ
    connection = pika.BlockingConnection(pika.ConnectionParameters('localhost'))
    channel = connection.channel()
    
    # Declare response queue
    result = channel.queue_declare(queue='', exclusive=True)
    callback_queue = result.method.queue
    
    def call_service(method, payload=None):
        # Create correlation ID
        corr_id = str(uuid.uuid4())
        
        # Publish message
        message = {
            'method': method,
            'payload': payload,
            'correlation_id': corr_id
        }
        
        channel.basic_publish(
            exchange='',
            routing_key='rabbitmesh.UserService-service',
            properties=pika.BasicProperties(
                reply_to=callback_queue,
                correlation_id=corr_id,
            ),
            body=json.dumps(message)
        )
        
        # Wait for response
        response = None
        def on_response(ch, method, props, body):
            nonlocal response
            if props.correlation_id == corr_id:
                response = json.loads(body.decode())
        
        channel.basic_consume(
            queue=callback_queue,
            on_message_callback=on_response,
            auto_ack=True
        )
        
        # Wait for response with timeout
        connection.process_data_events(time_limit=5)
        channel.stop_consuming()
        
        return response
    
    print("🧪 Testing macro-generated UserService")
    
    # Test ping
    print("Testing ping()...")
    response = call_service('ping')
    if response:
        print(f"✅ ping() -> {response}")
    else:
        print("❌ ping() -> No response")
    
    # Test get_user
    print("Testing get_user(42)...")
    response = call_service('get_user', 42)
    if response:
        print(f"✅ get_user(42) -> {response}")
    else:
        print("❌ get_user(42) -> No response")
    
    # Test create_user
    print("Testing create_user(Alice)...")
    response = call_service('create_user', {'name': 'Alice'})
    if response:
        print(f"✅ create_user(Alice) -> {response}")
    else:
        print("❌ create_user(Alice) -> No response")
    
    connection.close()
    print("🎉 All macro service tests completed!")

if __name__ == '__main__':
    test_service()