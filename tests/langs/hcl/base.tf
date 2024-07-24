// This is an example HCL configuration file for testing various software configurations

# Another line comment style

/* This is a multi-line comment block.
It starts with a slash and a star, and ends with a star and a slash.
*/

// Variable Definitions
variable "app_name" {
  description = "The name of the application"
  type        = string
  default     = "TestApp"
}

variable "instance_count" {
  description = "Number of instances to deploy"
  type        = number
  default     = 3
}

variable "enable_feature_x" {
  description = "Enable feature X"
  type        = bool
  default     = true
}

variable "admins" {
  description = "List of admin users"
  type        = list(string)
  default     = ["alice", "bob"]
}

// Locals
locals {
  app_env = "testing"
  version = "1.0.0"
}

// Providers
provider "aws" {
  region = "us-west-2"
}

// Data Sources
data "aws_ami" "latest_ubuntu" {
  most_recent = true
  owners      = ["self"]

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-bionic-18.04-amd64-server-*"]
  }
}

// Resources
resource "aws_instance" "app_server" {
  count         = var.instance_count
  ami           = data.aws_ami.latest_ubuntu.id
  instance_type = "t2.micro"
  tags = {
    Name        = "${var.app_name}-${count.index}"
    Environment = local.app_env
    Version     = local.version
  }

  // Dynamic block for user data
  dynamic "user_data" {
    for_each = var.enable_feature_x ? [1] : []
    content {
      data = <<EOF
#!/bin/bash
echo "Feature ${upper(var.app_name)} enabled"
EOF
    }
  }
}

// Output values
output "instance_ids" {
  description = "List of instance IDs"
  value       = aws_instance.app_server.*.id
}

output "admin_usernames" {
  description = "Admin usernames"
  value       = join(", ", var.admins)
}

// Modules
module "network" {
  source   = "./modules/network"
  vpc_cidr = "10.0.0.0/16"
}

// Conditional expression
resource "aws_s3_bucket" "app_bucket" {
  bucket = var.enable_feature_x ? "${var.app_name}-feature-x" : "${var.app_name}"
  acl    = "private"
}

// Using template expressions
resource "null_resource" "template_example" {
  provisioner "local-exec" {
    command = <<EOT
echo "Application: ${var.app_name}"
echo "Environment: ${local.app_env}"
EOT
  }
}

// Using for expressions
resource "aws_security_group" "web_sg" {
  name        = "${var.app_name}-web-sg"
  description = "Web security group for ${var.app_name}"
  vpc_id      = module.network.vpc_id

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

// Using nested blocks
resource "aws_iam_role" "app_role" {
  name = "${var.app_name}_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Action = "sts:AssumeRole",
        Effect = "Allow",
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })

  inline_policy {
    name = "app_policy"

    policy = jsonencode({
      Version = "2012-10-17",
      Statement = [
        {
          Action   = ["s3:ListBucket"],
          Effect   = "Allow",
          Resource = "*"
        }
      ]
    })
  }
}
